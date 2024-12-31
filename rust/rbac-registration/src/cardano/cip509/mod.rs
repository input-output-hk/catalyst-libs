//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix

pub mod rbac;
pub mod types;
pub mod utils;
pub(crate) mod validation;
pub mod x509_chunks;

use anyhow::anyhow;
use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{crypto::hash::Hash, ledger::traverse::MultiEraTx};
use strum_macros::FromRepr;
use tracing::error;
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
        decode_helper::{decode_bytes, decode_helper, decode_map_len},
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
#[derive(Debug, PartialEq, Clone)]
pub struct Cip509 {
    /// A registration purpose (`UUIDv4`).
    ///
    /// The purpose is defined by the consuming dApp.
    pub purpose: Uuid,
    /// Transaction inputs hash.
    pub txn_inputs_hash: TxInputHash,
    /// An optional hash of the previous transaction.
    ///
    /// The hash must always be present except for the first registration transaction.
    // TODO: Use the `Blake2b256Hash` type from the `cardano-blockchain-types` crate.
    pub prv_tx_id: Option<Hash<32>>,
    /// Metadata.
    ///
    /// This field encoded in chunks. See [`X509Chunks`] for more details.
    pub metadata: Cip509RbacMetadata,
    /// Validation signature.
    pub validation_signature: ValidationSignature,
}

/// Additional data from the CIP509 validation.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AdditionalData {
    /// Bytes of precomputed auxiliary data.
    pub precomputed_aux: Vec<u8>,
}

/// Enum of CIP509 metadatum with its associated unsigned integer value.
#[allow(clippy::module_name_repetitions)]
#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub(crate) enum Cip509IntIdentifier {
    /// Purpose.
    Purpose = 0,
    /// Transaction inputs hash.
    TxInputsHash = 1,
    /// Previous transaction ID.
    PreviousTxId = 2,
    /// Validation signature.
    ValidationSignature = 99,
}

impl Decode<'_, ()> for Cip509 {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP509")?;

        let mut purpose = Uuid::default();
        let mut txn_inputs_hash = TxInputHash::default();
        let mut prv_tx_id = None;
        let mut metadata = None;
        let mut validation_signature = Vec::new();

        for _ in 0..map_len {
            // Use probe to peak
            let key = d.probe().u8()?;
            if let Some(key) = Cip509IntIdentifier::from_repr(key) {
                // Consuming the int
                let _: u8 = decode_helper(d, "CIP509", ctx)?;
                match key {
                    Cip509IntIdentifier::Purpose => {
                        purpose = Uuid::try_from(decode_bytes(d, "CIP509 purpose")?)
                            .map_err(|_| decode::Error::message("Invalid data size of Purpose"))?;
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        txn_inputs_hash =
                            TxInputHash::try_from(decode_bytes(d, "CIP509 txn inputs hash")?)
                                .map_err(|_| {
                                    decode::Error::message("Invalid data size of TxInputsHash")
                                })?;
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        let hash: [u8; 32] = decode_bytes(d, "CIP509 previous tx ID")?
                            .try_into()
                            .map_err(|_| {
                            decode::Error::message("Invalid data size of PreviousTxId")
                        })?;
                        prv_tx_id = Some(Hash::from(hash));
                    },
                    Cip509IntIdentifier::ValidationSignature => {
                        let signature = decode_bytes(d, "CIP509 validation signature")?;
                        validation_signature = signature;
                    },
                }
            } else {
                // Handle the x509 chunks 10 11 12
                let x509_chunks = X509Chunks::decode(d, ctx)?;
                // Technically it is possible to store multiple copies (or different instances) of
                // metadata, but it isn't allowed. See this link for more details:
                // https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md#keys-10-11-or-12---x509-chunked-data
                if metadata.is_some() {
                    return Err(decode::Error::message(
                        "Only one instance of the chunked metadata should be present",
                    ));
                }
                metadata = Some(x509_chunks.into());
            }
        }

        let metadata =
            metadata.ok_or_else(|| decode::Error::message("Missing metadata in CIP509"))?;
        let validation_signature = validation_signature
            .try_into()
            .map_err(|e| decode::Error::message(format!("Invalid validation signature: {e:?}")))?;

        Ok(Self {
            purpose,
            txn_inputs_hash,
            prv_tx_id,
            metadata,
            validation_signature,
        })
    }
}

impl Cip509 {
    /// Performs the basic validation of CIP509.
    ///
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
    /// # Errors
    ///
    /// An error is returned if any of the validation steps is failed. The error contains
    /// the description of all failed steps.
    pub fn validate(self, txn: &MultiEraTx) -> anyhow::Result<(Self, AdditionalData)> {
        let mut validation_report = Vec::new();

        let is_valid_txn_inputs_hash =
            validate_txn_inputs_hash(&self, txn, &mut validation_report).unwrap_or(false);
        let (is_valid_aux, precomputed_aux) =
            validate_aux(txn, &mut validation_report).unwrap_or_default();
        let mut is_valid_stake_public_key = true;
        let mut is_valid_payment_key = true;
        let mut is_valid_signing_key = true;
        // Validate only role 0
        for role in &self.metadata.role_set {
            if role.role_number == 0 {
                is_valid_stake_public_key =
                    validate_stake_public_key(&self, txn, &mut validation_report).unwrap_or(false);
                is_valid_payment_key =
                    validate_payment_key(txn, role, &mut validation_report).unwrap_or(false);
                is_valid_signing_key = validate_role_singing_key(role, &mut validation_report);
            }
        }

        if is_valid_aux
            && is_valid_txn_inputs_hash
            && is_valid_stake_public_key
            && is_valid_payment_key
            && is_valid_signing_key
        {
            Ok((self, AdditionalData { precomputed_aux }))
        } else {
            let error = format!("CIP509 validation failed: {validation_report:?}");
            error!(error);
            Err(anyhow!(error))
        }
    }
}
