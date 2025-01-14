//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

use anyhow::anyhow;
use cardano_blockchain_types::{
    hashes::{Blake2b256Hash, BLAKE_2B256_SIZE},
    Slot, TxnIndex,
};
use catalyst_types::problem_report::ProblemReport;
use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{
    codec::utils::Nullable,
    ledger::traverse::{MultiEraBlock, MultiEraTx},
};
use strum_macros::FromRepr;
use tracing::warn;
use uuid::Uuid;

use crate::{
    cardano::{
        cip509::{
            decode_context::DecodeContext,
            rbac::Cip509RbacMetadata,
            types::{RoleNumber, TxInputHash, ValidationSignature},
            utils::Cip0134UriSet,
            validation::{
                validate_aux, validate_role_signing_key, validate_stake_public_key,
                validate_txn_inputs_hash,
            },
            x509_chunks::X509Chunks,
            RoleData,
        },
        transaction::raw_aux_data::RawAuxData,
    },
    utils::decode_helper::{
        decode_bytes, decode_helper, decode_map_len, report_duplicated_key, report_missing_keys,
    },
};

/// CIP509 label.
pub const LABEL: u64 = 509;

/// A x509 metadata envelope.
///
/// The envelope is required to prevent replayability attacks. See [this document] for
/// more details.
///
/// [this document]: https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct Cip509 {
    /// A registration purpose (`UUIDv4`).
    ///
    /// The purpose is defined by the consuming dApp.
    purpose: Option<Uuid>,
    /// Transaction inputs hash.
    txn_inputs_hash: Option<TxInputHash>,
    /// An optional hash of the previous transaction.
    ///
    /// The hash must always be present except for the first registration transaction.
    prv_tx_id: Option<Blake2b256Hash>,
    /// Metadata.
    ///
    /// This field encoded in chunks. See [`X509Chunks`] for more details.
    metadata: Option<Cip509RbacMetadata>,
    /// A validation signature.
    validation_signature: Option<ValidationSignature>,
    /// A report potentially containing all the issues occurred during `Cip509` decoding
    /// and validation.
    ///
    /// The data located in `Cip509` is only considered valid if
    /// `ProblemReport::is_problematic()` returns false.
    report: ProblemReport,
    /// A slot identifying the block that this `Cip509` was extracted from.
    slot: Slot,
    /// A transaction index.
    transaction_index: TxnIndex,
}

/// Enum of CIP509 metadatum with its associated unsigned integer value.
#[allow(clippy::module_name_repetitions)]
#[derive(FromRepr, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
enum Cip509IntIdentifier {
    /// Purpose.
    Purpose = 0,
    /// Transaction inputs hash.
    TxInputsHash = 1,
    /// Previous transaction ID.
    PreviousTxId = 2,
    /// Validation signature.
    ValidationSignature = 99,
}

impl Cip509 {
    /// Returns a `Cip509` instance if it is present in the given transaction, otherwise
    /// `None` is returned.
    ///
    /// # Errors
    ///
    /// An error is only returned if the data is completely corrupted. In all other cases
    /// the `Cip509` structure contains fully or partially decoded data.
    pub fn new(block: &MultiEraBlock, index: TxnIndex) -> Result<Option<Self>, anyhow::Error> {
        // Find the transaction and decode the relevant data.
        let transactions = block.txs();
        let transaction = transactions.get(usize::from(index)).ok_or_else(|| {
            anyhow!(
                "Invalid transaction index {index:?}, transactions count = {}",
                block.tx_count()
            )
        })?;

        let MultiEraTx::Conway(transaction) = transaction else {
            return Err(anyhow!("Unsupported era: {}", transaction.era()));
        };

        let auxiliary_data = match &transaction.auxiliary_data {
            Nullable::Some(v) => v.raw_cbor(),
            _ => return Ok(None),
        };
        let raw_auxiliary_data = RawAuxData::new(auxiliary_data);
        let Some(metadata) = raw_auxiliary_data.get_metadata(LABEL) else {
            return Ok(None);
        };

        let mut decoder = Decoder::new(metadata.as_slice());
        let mut report = ProblemReport::new("Decoding and validating Cip509");
        let mut decode_context = DecodeContext {
            slot: block.slot().into(),
            transaction_index: index,
            transaction: &transaction,
            report: &mut report,
        };
        let cip509 = match Cip509::decode(&mut decoder, &mut decode_context) {
            Ok(v) => v,
            Err(e) => {
                // We should get here only if we were unable to decode even the first byte.
                decode_context
                    .report
                    .other(&format!("{e:?}"), "Failed to decode Cip509");
                return Ok(Some(Self::with_decode_context(&decode_context)));
            },
        };

        // Perform the validation.
        if let Some(txn_inputs_hash) = &cip509.txn_inputs_hash {
            validate_txn_inputs_hash(txn_inputs_hash, transaction, &cip509.report);
        };
        validate_aux(
            auxiliary_data,
            transaction.transaction_body.auxiliary_data_hash.as_ref(),
            &cip509.report,
        );
        // The following checks are only performed for  the role 0.
        if let Some(role_data) = cip509.role_data(RoleNumber::ROLE_0) {
            validate_stake_public_key(transaction, cip509.certificate_uris(), &cip509.report);
            validate_role_signing_key(role_data, &cip509.report);
        }

        Ok(Some(cip509))
    }

    /// Returns a list of Cip509 instances from all the transactions of the given block.
    pub fn from_block(block: &MultiEraBlock) -> Vec<Self> {
        let mut result = Vec::new();

        for index in 0..block.tx_count() {
            let index = TxnIndex::from(index);
            match Self::new(block, index) {
                Ok(Some(v)) => result.push(v),
                // Normal situation: there is no Cip509 data in this transaction.
                Ok(None) => {},
                Err(e) => {
                    warn!(
                        "Unable to extract Cip509 from the {} block {index:?} transaction: {e:?}",
                        block.slot()
                    );
                },
            }
        }

        result
    }

    /// Creates an "empty" `Cip509` instance with all optional fields set to `None`.
    fn with_decode_context(context: &DecodeContext) -> Self {
        Self {
            purpose: None,
            txn_inputs_hash: None,
            prv_tx_id: None,
            metadata: None,
            validation_signature: None,
            report: context.report.clone(),
            slot: context.slot,
            transaction_index: context.transaction_index,
        }
    }

    /// Returns all role numbers present in this `Cip509` instance.
    pub fn all_roles(&self) -> Vec<RoleNumber> {
        if let Some(metadata) = &self.metadata {
            metadata.role_data.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a role data for the given role if it is present.
    pub fn role_data(&self, role: RoleNumber) -> Option<&RoleData> {
        self.metadata.as_ref().and_then(|m| m.role_data.get(&role))
    }

    /// Returns a hash of the previous transaction.
    pub fn previous_transaction(&self) -> Option<Blake2b256Hash> {
        self.prv_tx_id
    }

    /// Returns a problem report
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }

    /// Returns a slot and a transaction index where this data is originating from.
    pub fn origin(&self) -> (Slot, TxnIndex) {
        (self.slot, self.transaction_index)
    }

    /// Returns URIs contained in both x509 and c509 certificates of `Cip509` metadata.
    pub fn certificate_uris(&self) -> Option<&Cip0134UriSet> {
        self.metadata.as_ref().map(|m| &m.certificate_uris)
    }

    /// Returns a transaction inputs hash.
    pub fn txn_inputs_hash(&self) -> Option<&TxInputHash> {
        self.txn_inputs_hash.as_ref()
    }

    /// Returns `Cip509` fields consuming the structure if it was successfully decoded and
    /// validated otherwise return the problem report that contains all the encountered
    /// issues.
    ///
    /// # Errors
    ///
    /// - `Err(ProblemReport)`
    pub fn consume(self) -> Result<(Uuid, Cip509RbacMetadata), ProblemReport> {
        match (
            self.purpose,
            self.txn_inputs_hash,
            self.metadata,
            self.validation_signature,
        ) {
            (Some(purpose), Some(_), Some(metadata), Some(_)) if !self.report.is_problematic() => {
                Ok((purpose, metadata))
            },

            _ => Err(self.report),
        }
    }
}

impl Decode<'_, DecodeContext<'_, '_>> for Cip509 {
    fn decode(d: &mut Decoder, decode_context: &mut DecodeContext) -> Result<Self, decode::Error> {
        let context = "Decoding Cip509";

        // It is ok to return error here because we were unable to decode anything, but everywhere
        // below we should try to recover as much data as possible and not to return early.
        let map_len = decode_map_len(d, context)?;

        let mut result = Self::with_decode_context(&decode_context);

        let mut found_keys = Vec::new();
        let mut is_metadata_found = false;

        for index in 0..map_len {
            // We don't want to consume key here because it can be a part of chunked metadata that
            // is decoded below.
            let Ok(key) = d.probe().u8() else {
                result.report.other(
                    &format!("Unable to decode map key ({index} index)"),
                    context,
                );
                break;
            };
            if let Some(key) = Cip509IntIdentifier::from_repr(key) {
                // Consume the key. This should never fail because we used `probe` above.
                let _: u8 = decode_helper(d, context, &mut ())?;

                if report_duplicated_key(&found_keys, &key, index, context, &result.report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    Cip509IntIdentifier::Purpose => {
                        result.purpose = decode_purpose(d, context, &result.report);
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        result.txn_inputs_hash = decode_input_hash(d, context, &result.report);
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        result.prv_tx_id =
                            decode_previous_transaction_id(d, context, &result.report);
                    },
                    Cip509IntIdentifier::ValidationSignature => {
                        result.validation_signature =
                            decode_validation_signature(d, context, &result.report);
                    },
                }
            } else {
                // Handle the x509 chunks 10 11 12
                // Technically it is possible to store multiple copies (or different instances) of
                // metadata, but it isn't allowed. See this link for more details:
                // https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md#keys-10-11-or-12---x509-chunked-data
                if is_metadata_found {
                    result.report.duplicate_field(
                        "metadata",
                        "Only one instance of the chunked metadata should be present",
                        context,
                    );
                    continue;
                }
                is_metadata_found = true;

                match X509Chunks::decode(d, decode_context) {
                    Ok(chunks) => result.metadata = chunks.into(),
                    Err(e) => {
                        result.report.other(
                            &format!("Unable to decode metadata from chunks: {e:?}"),
                            context,
                        );
                    },
                };
            }
        }

        let required_keys = [
            Cip509IntIdentifier::Purpose,
            Cip509IntIdentifier::TxInputsHash,
            Cip509IntIdentifier::ValidationSignature,
        ];
        report_missing_keys(&found_keys, &required_keys, context, &result.report);
        if !is_metadata_found {
            result
                .report
                .missing_field("metadata (10, 11 or 12 chunks)", context);
        }

        Ok(result)
    }
}

/// Decodes purpose.
fn decode_purpose(d: &mut Decoder, context: &str, report: &ProblemReport) -> Option<Uuid> {
    let bytes = match decode_bytes(d, "Cip509 purpose") {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Unable to decode purpose: {e:?}"), context);
            return None;
        },
    };

    let len = bytes.len();
    if let Ok(v) = Uuid::try_from(bytes) {
        Some(v)
    } else {
        report.invalid_value(
            "purpose",
            &format!("{len} bytes"),
            "must be 16 bytes long",
            context,
        );
        None
    }
}

/// Decodes input hash.
fn decode_input_hash(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Option<TxInputHash> {
    let bytes = match decode_bytes(d, "Cip509 txn inputs hash") {
        Ok(v) => v,
        Err(e) => {
            report.other(
                &format!("Unable to decode transaction inputs hash: {e:?}"),
                context,
            );
            return None;
        },
    };

    let len = bytes.len();
    if let Ok(v) = TxInputHash::try_from(bytes) {
        Some(v)
    } else {
        report.invalid_value(
            "transaction inputs hash",
            &format!("{len} bytes"),
            "must be 16 bytes long",
            context,
        );
        None
    }
}

/// Decodes previous transaction id.
fn decode_previous_transaction_id(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Option<Blake2b256Hash> {
    let bytes = match decode_bytes(d, "Cip509 previous transaction id") {
        Ok(v) => v,
        Err(e) => {
            report.other(
                &format!("Unable to decode previous transaction id: {e:?}"),
                context,
            );
            return None;
        },
    };

    let len = bytes.len();
    if let Ok(v) = Blake2b256Hash::try_from(bytes) {
        Some(v)
    } else {
        report.invalid_value(
            "previous transaction hash",
            &format!("{len} bytes"),
            &format!("must be {BLAKE_2B256_SIZE} bytes long"),
            context,
        );
        None
    }
}

/// Decodes validation signature.
fn decode_validation_signature(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Option<ValidationSignature> {
    let bytes = match decode_bytes(d, "Cip509 validation signature") {
        Ok(v) => v,
        Err(e) => {
            report.other(
                &format!("Unable to decode validation signature: {e:?}"),
                context,
            );
            return None;
        },
    };

    let len = bytes.len();
    if let Ok(v) = ValidationSignature::try_from(bytes) {
        Some(v)
    } else {
        report.invalid_value(
            "validation signature",
            &format!("{len} bytes"),
            "must be at least 1 byte and at most 64 bytes long",
            context,
        );
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_1.block"))
            .expect("Failed to decode hex block.");
        let block = MultiEraBlock::decode(&block).unwrap();
        let index = TxnIndex::from(3);
        let res = Cip509::new(&block, index)
            .expect("Failed to get Cip509")
            .expect("There must be Cip509 in block");
        assert!(!res.report.is_problematic());
    }

    #[test]
    fn from_block() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_1.block"))
            .expect("Failed to decode hex block.");
        let block = MultiEraBlock::decode(&block).unwrap();
        let res = Cip509::from_block(&block);
        assert_eq!(1, res.len());
        assert!(!res.first().unwrap().report.is_problematic());
    }
}
