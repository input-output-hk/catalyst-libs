//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix

pub mod rbac;
pub mod types;
pub(crate) mod utils;
pub(crate) mod validation;
pub mod x509_chunks;

use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{crypto::hash::Hash, ledger::traverse::MultiEraTx};
use strum_macros::FromRepr;
use types::{tx_input_hash::TxInputHash, uuidv4::UuidV4};
use validation::{
    validate_aux, validate_payment_key, validate_role_singing_key, validate_stake_public_key,
    validate_txn_inputs_hash,
};
use x509_chunks::X509Chunks;

use super::transaction::witness::TxWitness;
use crate::utils::{
    decode_helper::{decode_bytes, decode_helper, decode_map_len},
    general::{decode_utf8, decremented_index},
    hashing::{blake2b_128, blake2b_256},
};

/// CIP509 label.
pub const LABEL: u64 = 509;

/// CIP509.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509 {
    /// `UUIDv4` Purpose .
    pub purpose: UuidV4, // (bytes .size 16)
    /// Transaction inputs hash.
    pub txn_inputs_hash: TxInputHash, // bytes .size 16
    /// Optional previous transaction ID.
    pub prv_tx_id: Option<Hash<32>>, // bytes .size 32
    /// x509 chunks.
    pub x509_chunks: X509Chunks, // chunk_type => [ + x509_chunk ]
    /// Validation signature.
    pub validation_signature: Vec<u8>, // bytes size (1..64)
}

/// Validation value for CIP509 metadatum.
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509Validation {
    /// Boolean value for the validity of the transaction inputs hash.
    pub valid_txn_inputs_hash: bool,
    /// Boolean value for the validity of the auxiliary data.
    pub valid_aux: bool,
    /// Boolean value for the validity of the public key.
    pub valid_public_key: bool,
    /// Boolean value for the validity of the payment key.
    pub valid_payment_key: bool,
    /// Boolean value for the validity of the signing key.
    pub signing_key: bool,
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
        let mut cip509_metadatum = Cip509::default();
        for _ in 0..map_len {
            // Use probe to peak
            let key = d.probe().u8()?;
            if let Some(key) = Cip509IntIdentifier::from_repr(key) {
                // Consuming the int
                let _: u8 = decode_helper(d, "CIP509", ctx)?;
                match key {
                    Cip509IntIdentifier::Purpose => {
                        cip509_metadatum.purpose =
                            UuidV4::try_from(decode_bytes(d, "CIP509 purpose")?).map_err(|_| {
                                decode::Error::message("Invalid data size of Purpose")
                            })?;
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        cip509_metadatum.txn_inputs_hash =
                            TxInputHash::try_from(decode_bytes(d, "CIP509 txn inputs hash")?)
                                .map_err(|_| {
                                    decode::Error::message("Invalid data size of TxInputsHash")
                                })?;
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        let prv_tx_hash: [u8; 32] = decode_bytes(d, "CIP509 previous tx ID")?
                            .try_into()
                            .map_err(|_| {
                                decode::Error::message("Invalid data size of PreviousTxId")
                            })?;
                        cip509_metadatum.prv_tx_id = Some(Hash::from(prv_tx_hash));
                    },
                    Cip509IntIdentifier::ValidationSignature => {
                        let validation_signature = decode_bytes(d, "CIP509 validation signature")?;
                        if validation_signature.is_empty() || validation_signature.len() > 64 {
                            return Err(decode::Error::message(
                                "Invalid data size of ValidationSignature",
                            ));
                        }
                        cip509_metadatum.validation_signature = validation_signature;
                    },
                }
            } else {
                // Handle the x509 chunks 10 11 12
                let x509_chunks = X509Chunks::decode(d, ctx)?;
                cip509_metadatum.x509_chunks = x509_chunks;
            }
        }
        Ok(cip509_metadatum)
    }
}

impl Cip509 {
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
        let tx_input_validate =
            validate_txn_inputs_hash(self, txn, validation_report).unwrap_or(false);
        let (aux_validate, precomputed_aux) =
            validate_aux(txn, validation_report).unwrap_or_default();
        let mut stake_key_validate = true;
        let mut payment_key_validate = true;
        let mut signing_key = true;
        // Validate the role 0
        if let Some(role_set) = &self.x509_chunks.0.role_set {
            // Validate only role 0
            for role in role_set {
                if role.role_number == 0 {
                    stake_key_validate =
                        validate_stake_public_key(self, txn, validation_report).unwrap_or(false);
                    payment_key_validate =
                        validate_payment_key(txn, role, validation_report).unwrap_or(false);
                    signing_key = validate_role_singing_key(role, validation_report);
                }
            }
        }
        Cip509Validation {
            valid_txn_inputs_hash: tx_input_validate,
            valid_aux: aux_validate,
            valid_public_key: stake_key_validate,
            valid_payment_key: payment_key_validate,
            signing_key,
            additional_data: AdditionalData { precomputed_aux },
        }
    }
}
