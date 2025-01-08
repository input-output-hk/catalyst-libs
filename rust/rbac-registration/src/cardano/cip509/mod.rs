//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix

pub mod rbac;
pub mod types;
pub mod utils;
pub(crate) mod validation;
pub mod x509_chunks;

use cardano_blockchain_types::hashes::{Blake2b256Hash, BLAKE_2B256_SIZE};
use catalyst_types::problem_report::ProblemReport;
use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::ledger::traverse::MultiEraTx;
use strum_macros::FromRepr;
use types::tx_input_hash::TxInputHash;
use uuid::Uuid;
use validation::{
    validate_aux, validate_payment_key, validate_role_singing_key, validate_stake_public_key,
    validate_txn_inputs_hash,
};
use x509_chunks::X509Chunks;

use super::transaction::witness::TxWitness;
use crate::{
    cardano::cip509::{rbac::Cip509RbacMetadata, types::ValidationSignature},
    utils::{
        decode_helper::{
            decode_bytes, decode_helper, decode_map_len, report_duplicated_key, report_missing_keys,
        },
        general::decremented_index,
        hashing::{blake2b_128, blake2b_256},
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
#[derive(Debug, PartialEq, Clone, Default)]
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
    /// Validation signature.
    validation_signature: Option<ValidationSignature>,
}

/// Validation value for CIP509 metadatum.
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509Validation {
    /// Boolean value for the validity of the transaction inputs hash.
    pub is_valid_txn_inputs_hash: bool,
    /// Boolean value for the validity of the auxiliary data.
    pub is_valid_aux: bool,
    /// Boolean value for the validity of the stake public key.
    pub is_valid_stake_public_key: bool,
    /// Boolean value for the validity of the payment key.
    pub is_valid_payment_key: bool,
    /// Boolean value for the validity of the signing key.
    pub is_valid_signing_key: bool,
    /// Additional data from the CIP509 validation..
    pub additional_data: AdditionalData,
}

/// Additional data from the CIP509 validation.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AdditionalData {
    /// Bytes of precomputed auxiliary data.
    pub precomputed_aux: Vec<u8>,
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
    // TODO: FIXME: Replace validation with construction from `MultiEraBlock` and `TxnIndex`.
    // (https://github.com/input-output-hk/catalyst-libs/pull/127#discussion_r1901549418)

    /// Basic validation for CIP509
    /// The validation include the following:
    /// * Hashing the transaction inputs within the transaction should match the
    ///   txn-inputs-hash
    /// * Auxiliary data hash within the transaction should match the hash of the
    ///   auxiliary data itself.
    /// * Public key validation for role 0 where public key extracted from x509 and c509
    ///   subject alternative name should match one of the witness in witness set within
    ///   the transaction.
    /// * Payment key reference validation for role 0 where the reference should be either
    ///     1. Negative index reference - reference to transaction output in transaction:
    ///        should match some of the key within witness set.
    ///     2. Positive index reference - reference to the transaction input in
    ///        transaction: only check whether the index exist within the transaction
    ///        inputs.
    /// * Role signing key validation for role 0 where the signing keys should only be the
    ///   certificates
    ///
    ///  See:
    /// * <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
    /// * <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>
    ///
    /// Note: This CIP509 is still under development and is subject to change.
    ///
    /// # Parameters
    /// * `txn` - Transaction data was attached to and to be validated/decoded against.
    /// * `validation_report` - Validation report to store the validation result.
    pub fn validate(
        &self, txn: &MultiEraTx, validation_report: &mut Vec<String>,
    ) -> Cip509Validation {
        let is_valid_txn_inputs_hash =
            validate_txn_inputs_hash(self, txn, validation_report).unwrap_or(false);
        let (is_valid_aux, precomputed_aux) =
            validate_aux(txn, validation_report).unwrap_or_default();
        let mut is_valid_stake_public_key = true;
        let mut is_valid_payment_key = true;
        let mut is_valid_signing_key = true;
        // Validate only role 0
        for role in &self.metadata.role_set {
            if role.role_number == 0 {
                is_valid_stake_public_key =
                    validate_stake_public_key(self, txn, validation_report).unwrap_or(false);
                is_valid_payment_key =
                    validate_payment_key(txn, role, validation_report).unwrap_or(false);
                is_valid_signing_key = validate_role_singing_key(role, validation_report);
            }
        }
        Cip509Validation {
            is_valid_txn_inputs_hash,
            is_valid_aux,
            is_valid_stake_public_key,
            is_valid_payment_key,
            is_valid_signing_key,
            additional_data: AdditionalData { precomputed_aux },
        }
    }

    /// Returns a registration purpose.
    #[must_use]
    pub fn purpose(&self) -> Option<Uuid> {
        self.purpose
    }

    /// Returns an identifier of the previous transaction.
    #[must_use]
    pub fn previous_transaction(&self) -> Option<&Blake2b256Hash> {
        self.prv_tx_id.as_ref()
    }

    /// Returns CIP509 metadata.
    #[must_use]
    pub fn metadata(&self) -> Option<&Cip509RbacMetadata> {
        self.metadata.as_ref()
    }
}

impl Decode<'_, ProblemReport> for Cip509 {
    fn decode(d: &mut Decoder, report: &mut ProblemReport) -> Result<Self, decode::Error> {
        let context = "Cip509";
        let map_len = decode_map_len(d, context)?;

        let mut result = Self::default();
        let mut found_keys = Vec::new();
        let mut is_metadata_found = false;

        for index in 0..map_len {
            // Use probe to peak
            let key = d.probe().u8()?;
            if let Some(key) = Cip509IntIdentifier::from_repr(key) {
                // Consuming the int
                let _: u8 = decode_helper(d, context, &mut ())?;

                if report_duplicated_key(&found_keys, &key, index, context, report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    Cip509IntIdentifier::Purpose => {
                        result.purpose = decode_purpose(d, context, report)?;
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        result.txn_inputs_hash = decode_input_hash(d, context, report)?;
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        result.prv_tx_id = decode_previous_transaction_id(d, context, report)?;
                    },
                    Cip509IntIdentifier::ValidationSignature => {
                        result.validation_signature =
                            decode_validation_signature(d, context, report)?;
                    },
                }
            } else {
                // Handle the x509 chunks 10 11 12
                // Technically it is possible to store multiple copies (or different instances) of
                // metadata, but it isn't allowed. See this link for more details:
                // https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md#keys-10-11-or-12---x509-chunked-data
                if is_metadata_found {
                    report.duplicate_field(
                        "metadata",
                        "Only one instance of the chunked metadata should be present",
                        context,
                    );
                    continue;
                }
                is_metadata_found = true;

                let x509_chunks = X509Chunks::decode(d, report)?;
                result.metadata = x509_chunks.into();
            }
        }

        let required_keys = [
            Cip509IntIdentifier::Purpose,
            Cip509IntIdentifier::TxInputsHash,
            Cip509IntIdentifier::PreviousTxId,
            Cip509IntIdentifier::ValidationSignature,
        ];
        report_missing_keys(&found_keys, &required_keys, context, report);
        if !is_metadata_found {
            report.missing_field("metadata (10, 11 or 12 chunks)", context);
        }

        Ok(result)
    }
}

fn decode_purpose(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Result<Option<Uuid>, decode::Error> {
    let bytes = decode_bytes(d, "Cip509 purpose")?;
    let len = bytes.len();
    match Uuid::try_from(bytes) {
        Ok(v) => Ok(Some(v)),
        Err(_) => {
            report.invalid_value(
                "purpose",
                &format!("{len} bytes"),
                "must be 16 bytes long",
                context,
            );
            Ok(None)
        },
    }
}

fn decode_input_hash(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Result<Option<TxInputHash>, decode::Error> {
    let bytes = decode_bytes(d, "Cip509 txn inputs hash")?;
    let len = bytes.len();
    match TxInputHash::try_from(bytes) {
        Ok(v) => Ok(Some(v)),
        Err(_) => {
            report.invalid_value(
                "transaction inputs hash",
                &format!("{len} bytes"),
                "must be 16 bytes long",
                context,
            );
            Ok(None)
        },
    }
}

fn decode_previous_transaction_id(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Result<Option<Blake2b256Hash>, decode::Error> {
    let bytes = decode_bytes(d, "Cip509 previous transaction id")?;
    let len = bytes.len();
    match Blake2b256Hash::try_from(bytes) {
        Ok(v) => Ok(Some(v)),
        Err(_) => {
            report.invalid_value(
                "previous transaction hash",
                &format!("{len} bytes"),
                &format!("must be {BLAKE_2B256_SIZE} bytes long"),
                context,
            );
            Ok(None)
        },
    }
}

fn decode_validation_signature(
    d: &mut Decoder, context: &str, report: &ProblemReport,
) -> Result<Option<ValidationSignature>, decode::Error> {
    let bytes = decode_bytes(d, "Cip509 validation signature")?;
    let len = bytes.len();
    match ValidationSignature::try_from(bytes) {
        Ok(v) => Ok(Some(v)),
        Err(_) => {
            report.invalid_value(
                "validation signature",
                &format!("{len} bytes"),
                &format!("must be at least 1 byte and at most 64 bytes long"),
                context,
            );
            Ok(None)
        },
    }
}
