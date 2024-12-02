//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix

pub mod rbac;
pub mod utils;
pub mod x509_chunks;

mod decode_helper;

use std::sync::Arc;

use c509_certificate::{general_names::general_name::GeneralNameValue, C509ExtensionType};
use decode_helper::{decode_bytes, decode_helper, decode_map_len};
use der_parser::der::parse_der_sequence;
use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::traverse::MultiEraTxWithRawAuxiliary,
};
use rbac::{certs::C509Cert, role_data::RoleData};
use strum::FromRepr;
use utils::{
    compare_key_hash, decremented_index, extract_cip19_hash, extract_key_hash,
    zero_out_last_n_bytes,
};
use x509_cert::{der::Decode as _, ext::pkix::ID_CE_SUBJECT_ALT_NAME};
use x509_chunks::X509Chunks;

use super::{
    raw_aux_data::RawAuxData, DecodedMetadata, DecodedMetadataItem, DecodedMetadataValues,
    ValidationReport,
};
use crate::{
    utils::{blake2b_128, blake2b_256, decode_utf8},
    witness::TxWitness,
};

/// CIP509 label.
pub const LABEL: u64 = 509;

/// Context-specific primitive type with tag number 6 (`raw_tag` 134) for
/// uniform resource identifier (URI) in the subject alternative name extension.
pub(crate) const URI: u8 = 134;

/// CIP509 metadatum.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509 {
    /// `UUIDv4` Purpose .
    pub purpose: [u8; 16], // (bytes .size 16)
    /// Transaction inputs hash.
    pub txn_inputs_hash: [u8; 16], // bytes .size 16
    /// Optional previous transaction ID.
    pub prv_tx_id: Option<[u8; 32]>, // bytes .size 32
    /// x509 chunks.
    pub x509_chunks: X509Chunks, // chunk_type => [ + x509_chunk ]
    /// Validation signature.
    pub validation_signature: Vec<u8>, // bytes size (1..64)
    /// Validation value, not a part of CIP509, justs storing validity of the data.
    pub validation: Cip509Validation,
}

/// Validation value for CIP509 metadatum.
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509Validation {
    /// Boolean value for the validity of the transaction inputs hash.
    pub valid_txn_inputs_hash: bool,
    /// Boolean value for the validity of the auxiliary data.
    pub valid_aux: bool,
    /// Bytes of precomputed auxiliary data.
    pub precomputed_aux: Vec<u8>,
    /// Boolean value for the validity of the public key.
    pub valid_public_key: bool,
    /// Boolean value for the validity of the payment key.
    pub valid_payment_key: bool,
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
                        cip509_metadatum.purpose = decode_bytes(d, "CIP509 purpose")?
                            .try_into()
                            .map_err(|_| decode::Error::message("Invalid data size of Purpose"))?;
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        cip509_metadatum.txn_inputs_hash =
                            decode_bytes(d, "CIP509 txn inputs hash")?
                                .try_into()
                                .map_err(|_| {
                                    decode::Error::message("Invalid data size of TxInputsHash")
                                })?;
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        cip509_metadatum.prv_tx_id = Some(
                            decode_bytes(d, "CIP509 previous tx ID")?
                                .try_into()
                                .map_err(|_| {
                                    decode::Error::message("Invalid data size of PreviousTxId")
                                })?,
                        );
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

#[allow(clippy::module_name_repetitions)]
impl Cip509 {
    /// Decode and validate CIP509 Metadata
    ///
    /// The validation include the following:
    /// * Hashing the transaction inputs within the transaction should match the
    ///   txn-inputs-hash
    /// * Auxiliary data hash within the transaction should the hash of the auxiliary data
    ///   itself. This also includes logging the pre-computed hash where the last 64 bytes
    ///   are set to zero.
    /// * Public key validation for role 0 where public key extracted from x509 and c509
    ///   subject alternative name should match one of the witness in witness set within
    ///   the transaction.
    /// * Payment key reference validation for role 0 where the reference should be either
    ///     1. Negative index reference - reference to transaction output in transaction:
    ///        should match some of the key within witness set.
    ///     2. Positive index reference - reference to the transaction input in
    ///        transaction: only check whether the index exist within the transaction
    ///        inputs.
    ///
    /// See:
    /// * <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
    /// * <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>
    ///
    /// Note: This CIP509 is still under development and is subject to change.
    ///
    /// # Parameters
    /// * `decoded_metadata` - Decoded Metadata - Will be updated only if CIP509 Metadata
    ///   is found.
    /// * `txn` - Transaction data was attached to and to be validated/decoded against.
    /// * `raw_aux_data` - Raw Auxiliary Data for the transaction.
    /// * `txn_idx` - Transaction Index
    ///
    /// # Returns
    ///
    /// Nothing.  IF CIP509 Metadata is found it will be updated in `decoded_metadata`.
    pub(crate) fn decode_and_validate(
        decoded_metadata: &DecodedMetadata, txn: &MultiEraTxWithRawAuxiliary,
        raw_aux_data: &RawAuxData, txn_idx: usize,
    ) {
        // Get the CIP509 metadata if possible
        let Some(k509) = raw_aux_data.get_metadata(LABEL) else {
            return;
        };

        let mut validation_report = ValidationReport::new();
        let mut decoder = Decoder::new(k509.as_slice());

        let mut cip509 = match Cip509::decode(&mut decoder, &mut ()) {
            Ok(metadata) => metadata,
            Err(e) => {
                Cip509::default().validation_failure(
                    &format!("Failed to decode CIP509 metadata: {e}"),
                    &mut validation_report,
                    decoded_metadata,
                );
                return;
            },
        };

        // Validate transaction inputs hash
        match cip509.validate_txn_inputs_hash(txn, &mut validation_report, decoded_metadata) {
            Some(b) => cip509.validation.valid_txn_inputs_hash = b,
            None => {
                cip509.validation_failure(
                    "Failed to validate transaction inputs hash",
                    &mut validation_report,
                    decoded_metadata,
                );
            },
        }

        // Validate the auxiliary data
        match cip509.validate_aux(txn, &mut validation_report, decoded_metadata) {
            Some(b) => cip509.validation.valid_aux = b,
            None => {
                cip509.validation_failure(
                    "Failed to validate auxiliary data",
                    &mut validation_report,
                    decoded_metadata,
                );
            },
        }

        // Validate the role 0
        if let Some(role_set) = &cip509.x509_chunks.0.role_set {
            // Validate only role 0
            for role in role_set {
                if role.role_number == 0 {
                    // Validate stake public key to in certificate to the witness set in transaction
                    match cip509.validate_stake_public_key(
                        txn,
                        &mut validation_report,
                        decoded_metadata,
                        txn_idx,
                    ) {
                        Some(b) => cip509.validation.valid_public_key = b,
                        None => {
                            cip509.validation_failure(
                                &format!("Failed to validate stake public key in tx id {txn_idx}"),
                                &mut validation_report,
                                decoded_metadata,
                            );
                        },
                    }
                    // Validate payment key reference
                    match cip509.validate_payment_key(
                        txn,
                        &mut validation_report,
                        decoded_metadata,
                        txn_idx,
                        role,
                    ) {
                        Some(b) => cip509.validation.valid_payment_key = b,
                        None => {
                            cip509.validation_failure(
                                &format!("Failed to validate payment key in tx id {txn_idx}"),
                                &mut validation_report,
                                decoded_metadata,
                            );
                        },
                    }
                }
            }
        }
    }

    /// Handle validation failure.
    fn validation_failure(
        &self, reason: &str, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) {
        validation_report.push(reason.into());
        decoded_metadata.0.insert(
            LABEL,
            Arc::new(DecodedMetadataItem {
                value: DecodedMetadataValues::Cip509(Arc::new(self.clone()).clone()),
                report: validation_report.clone(),
            }),
        );
    }

    /// Transaction inputs hash validation.
    /// Must exist and match the hash of the transaction inputs.
    fn validate_txn_inputs_hash(
        &self, txn: &MultiEraTxWithRawAuxiliary, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) -> Option<bool> {
        let mut buffer = Vec::new();
        let mut e = Encoder::new(&mut buffer);
        match txn {
            MultiEraTxWithRawAuxiliary::AlonzoCompatible(tx, _) => {
                let inputs = tx.transaction_body.inputs.clone();
                if let Err(e) = e.array(inputs.len() as u64) {
                    self.validation_failure(
                        &format!("Failed to encode array of transaction input in validate_txn_inputs_hash: {e}"),
                        validation_report,
                        decoded_metadata,
                    );
                    return None;
                }
                for input in &inputs {
                    if let Err(e) = input.encode(&mut e, &mut ()) {
                        self.validation_failure(
                            &format!("Failed to encode transaction input in validate_txn_inputs_hash: {e}"),
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                }
            },
            MultiEraTxWithRawAuxiliary::Babbage(tx) => {
                let inputs = tx.transaction_body.inputs.clone();
                if let Err(e) = e.array(inputs.len() as u64) {
                    self.validation_failure(
                        &format!("Failed to encode array of transaction input in validate_txn_inputs_hash: {e}"),
                        validation_report,
                        decoded_metadata,
                    );
                    return None;
                }
                for input in &inputs {
                    if let Err(e) = input.encode(&mut e, &mut ()) {
                        self.validation_failure(
                            &format!("Failed to encode transaction input in validate_txn_inputs_hash: {e}"),
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                }
            },
            MultiEraTxWithRawAuxiliary::Conway(tx) => {
                let inputs = tx.transaction_body.inputs.clone();
                if let Err(e) = e.array(inputs.len() as u64) {
                    self.validation_failure(
                        &format!("Failed to encode array of transaction input in validate_txn_inputs_hash: {e}"),
                        validation_report,
                        decoded_metadata,
                    );
                    return None;
                }
                for input in &inputs {
                    match input.encode(&mut e, &mut ()) {
                        Ok(()) => {},
                        Err(e) => {
                            self.validation_failure(
                                &format!(
                                "Failed to encode transaction input in validate_txn_inputs_hash {e}"
                            ),
                                validation_report,
                                decoded_metadata,
                            );
                            return None;
                        },
                    }
                }
            },
            _ => {
                self.validation_failure(
                    "Unsupported transaction era for transaction inputs hash validation",
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        }
        let inputs_hash = match blake2b_128(&buffer) {
            Ok(hash) => hash,
            Err(e) => {
                self.validation_failure(
                    &format!("Failed to hash transaction inputs in validate_txn_inputs_hash {e}"),
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        };
        Some(inputs_hash == self.txn_inputs_hash)
    }

    /// Validate the auxiliary data with the auxiliary data hash in the transaction.
    /// Also log out the pre-computed hash where the validation signature (99) set to
    /// zero.
    fn validate_aux(
        &mut self, txn: &MultiEraTxWithRawAuxiliary, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) -> Option<bool> {
        match txn {
            MultiEraTxWithRawAuxiliary::AlonzoCompatible(tx, _) => {
                if let pallas::codec::utils::Nullable::Some(a) = &tx.auxiliary_data {
                    let original_aux = a.raw_cbor();
                    let aux_data_hash =
                        tx.transaction_body
                            .auxiliary_data_hash
                            .as_ref()
                            .or_else(|| {
                                self.validation_failure(
                                    "Auxiliary data hash not found in transaction",
                                    validation_report,
                                    decoded_metadata,
                                );
                                None
                            })?;
                    self.validate_aux_helper(
                        original_aux,
                        aux_data_hash,
                        validation_report,
                        decoded_metadata,
                    )
                } else {
                    self.validation_failure(
                        "Auxiliary data not found in transaction",
                        validation_report,
                        decoded_metadata,
                    );
                    None
                }
            },
            MultiEraTxWithRawAuxiliary::Babbage(tx) => {
                if let pallas::codec::utils::Nullable::Some(a) = &tx.auxiliary_data {
                    let original_aux = a.raw_cbor();
                    let aux_data_hash =
                        tx.transaction_body
                            .auxiliary_data_hash
                            .as_ref()
                            .or_else(|| {
                                self.validation_failure(
                                    "Auxiliary data hash not found in transaction",
                                    validation_report,
                                    decoded_metadata,
                                );
                                None
                            })?;
                    self.validate_aux_helper(
                        original_aux,
                        aux_data_hash,
                        validation_report,
                        decoded_metadata,
                    )
                } else {
                    self.validation_failure(
                        "Auxiliary data not found in transaction",
                        validation_report,
                        decoded_metadata,
                    );
                    None
                }
            },
            MultiEraTxWithRawAuxiliary::Conway(tx) => {
                if let pallas::codec::utils::Nullable::Some(a) = &tx.auxiliary_data {
                    let original_aux = a.raw_cbor();
                    let aux_data_hash =
                        tx.transaction_body
                            .auxiliary_data_hash
                            .as_ref()
                            .or_else(|| {
                                self.validation_failure(
                                    "Auxiliary data hash not found in transaction",
                                    validation_report,
                                    decoded_metadata,
                                );
                                None
                            })?;
                    self.validate_aux_helper(
                        original_aux,
                        aux_data_hash,
                        validation_report,
                        decoded_metadata,
                    )
                } else {
                    self.validation_failure(
                        "Auxiliary data not found in transaction",
                        validation_report,
                        decoded_metadata,
                    );
                    None
                }
            },
            _ => {
                self.validation_failure(
                    "Unsupported transaction era for auxiliary data validation",
                    validation_report,
                    decoded_metadata,
                );
                None
            },
        }
    }

    /// Helper function for auxiliary data validation.
    fn validate_aux_helper(
        &mut self, original_aux: &[u8], aux_data_hash: &Bytes,
        validation_report: &mut ValidationReport, decoded_metadata: &DecodedMetadata,
    ) -> Option<bool> {
        let mut vec_aux = original_aux.to_vec();

        // Zero out the last 64 bytes
        zero_out_last_n_bytes(&mut vec_aux, 64);

        // Pre-computed aux with the last 64 bytes set to zero
        self.validation.precomputed_aux = vec_aux;

        // Compare the hash
        match blake2b_256(original_aux) {
            Ok(original_hash) => {
                return Some(aux_data_hash.as_ref() == original_hash);
            },
            Err(e) => {
                self.validation_failure(
                    &format!("Cannot hash auxiliary data {e}"),
                    validation_report,
                    decoded_metadata,
                );
                None
            },
        }
    }

    /// Validate the stake public key in the certificate with witness set in transaction.
    #[allow(clippy::too_many_lines)]
    fn validate_stake_public_key(
        &self, txn: &MultiEraTxWithRawAuxiliary, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, txn_idx: usize,
    ) -> Option<bool> {
        let mut pk_addrs = Vec::new();
        match txn {
            MultiEraTxWithRawAuxiliary::AlonzoCompatible(..)
            | MultiEraTxWithRawAuxiliary::Babbage(_)
            | MultiEraTxWithRawAuxiliary::Conway(_) => {
                // X509 certificate
                if let Some(x509_certs) = &self.x509_chunks.0.x509_certs {
                    for cert in x509_certs {
                        // Attempt to decode the DER certificate
                        let der_cert = match x509_cert::Certificate::from_der(&cert.0) {
                            Ok(cert) => cert,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to decode x509 certificate DER: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };

                        // Find the Subject Alternative Name extension
                        let san_ext =
                            der_cert
                                .tbs_certificate
                                .extensions
                                .as_ref()
                                .and_then(|exts| {
                                    exts.iter()
                                        .find(|ext| ext.extn_id == ID_CE_SUBJECT_ALT_NAME)
                                });

                        // Subject Alternative Name extension if it exists
                        if let Some(san_ext) = san_ext {
                            match parse_der_sequence(san_ext.extn_value.as_bytes()) {
                                Ok((_, parsed_seq)) => {
                                    for data in parsed_seq.ref_iter() {
                                        // Check for context-specific primitive type with tag number
                                        // 6 (raw_tag 134)
                                        if data.header.raw_tag() == Some(&[URI]) {
                                            match data.content.as_slice() {
                                                Ok(content) => {
                                                    // Decode the UTF-8 string
                                                    let addr: String = match decode_utf8(content) {
                                                        Ok(addr) => addr,
                                                        Err(e) => {
                                                            self.validation_failure(
                                                                &format!(
                                                                    "Failed to decode UTF-8 string for context-specific primitive type with raw tag 134: {e}",
                                                                ),
                                                                validation_report,
                                                                decoded_metadata,
                                                            );
                                                            return None;
                                                        },
                                                    };

                                                    // Extract the CIP19 hash and push into
                                                    // array
                                                    if let Some(h) =
                                                        extract_cip19_hash(&addr, Some("stake"))
                                                    {
                                                        pk_addrs.push(h);
                                                    }
                                                },
                                                Err(e) => {
                                                    self.validation_failure(
                                                        &format!("Failed to process content for context-specific primitive type with raw tag 134: {e}"),
                                                        validation_report,
                                                        decoded_metadata,
                                                    );
                                                    return None;
                                                },
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    self.validation_failure(
                                        &format!(
                                            "Failed to parse DER sequence for Subject Alternative Name extension: {e}",
                                        ),
                                        validation_report,
                                        decoded_metadata,
                                    );
                                    return None;
                                },
                            }
                        }
                    }
                }
                // C509 Certificate
                if let Some(c509_certs) = &self.x509_chunks.0.c509_certs {
                    for cert in c509_certs {
                        match cert {
                            C509Cert::C509CertInMetadatumReference(_) => {
                                self.validation_failure(
                                    "C509 metadatum reference is currently not supported",
                                    validation_report,
                                    decoded_metadata,
                                );
                            },
                            C509Cert::C509Certificate(c509) => {
                                for exts in c509.tbs_cert().extensions().extensions() {
                                    if *exts.registered_oid().c509_oid().oid()
                                        == C509ExtensionType::SubjectAlternativeName.oid()
                                    {
                                        match exts.value() {
                                            c509_certificate::extensions::extension::ExtensionValue::AlternativeName(alt_name) => {
                                                match alt_name.general_name() {
                                                    c509_certificate::extensions::alt_name::GeneralNamesOrText::GeneralNames(gn) => {
                                                        for name in gn.general_names() {
                                                            if name.gn_type() == &c509_certificate::general_names::general_name::GeneralNameTypeRegistry::UniformResourceIdentifier {
                                                                match name.gn_value() {
                                                                    GeneralNameValue::Text(s) => {
                                                                            if let Some(h) = extract_cip19_hash(s, Some("stake")) {
                                                                                pk_addrs.push(h);
                                                                            }
                                                                    },
                                                                    _ => {
                                                                        self.validation_failure(
                                                                            "Failed to get the value of subject alternative name",
                                                                            validation_report,
                                                                            decoded_metadata,
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    },
                                                    c509_certificate::extensions::alt_name::GeneralNamesOrText::Text(_) => {
                                                        self.validation_failure(
                                                            "Failed to find C509 general names in subject alternative name",
                                                            validation_report,
                                                            decoded_metadata,
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {
                                                self.validation_failure(
                                                    "Failed to get C509 subject alternative name",
                                                    validation_report,
                                                    decoded_metadata,
                                                );
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            },
            _ => {
                self.validation_failure(
                    "Unsupported transaction era for public key validation",
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        }

        // Create TxWitness
        let witnesses = match TxWitness::new(&[txn.clone()]) {
            Ok(witnesses) => witnesses,
            Err(e) => {
                self.validation_failure(
                    &format!("Failed to create TxWitness: {e}"),
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        };

        let index = match u16::try_from(txn_idx) {
            Ok(value) => value,
            Err(e) => {
                self.validation_failure(
                    &format!("Failed to convert transaction index to usize: {e}"),
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        };
        Some(
            compare_key_hash(&pk_addrs, &witnesses, index)
                .map_err(|e| {
                    self.validation_failure(
                        &format!("Failed to compare public keys with witnesses {e}"),
                        validation_report,
                        decoded_metadata,
                    );
                })
                .is_ok(),
        )
    }

    /// Validate the payment key
    #[allow(clippy::too_many_lines)]
    fn validate_payment_key(
        &self, txn: &MultiEraTxWithRawAuxiliary, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, txn_idx: usize, role_data: &RoleData,
    ) -> Option<bool> {
        if let Some(payment_key) = role_data.payment_key {
            // 0 reference key is not allowed
            if payment_key == 0 {
                self.validation_failure(
                    "Invalid payment reference key, 0 is not allowed",
                    validation_report,
                    decoded_metadata,
                );
                return None;
            }
            match txn {
                MultiEraTxWithRawAuxiliary::AlonzoCompatible(tx, _) => {
                    // Handle negative payment keys (reference to tx output)
                    if payment_key < 0 {
                        let witness = match TxWitness::new(&[txn.clone()]) {
                            Ok(witness) => witness,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to create TxWitness: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };
                        let index = match decremented_index(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to get index of payment key: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };
                        let outputs = tx.transaction_body.outputs.clone();
                        if let Some(output) = outputs.get(index) {
                            return self.validate_payment_output_key_helper(
                                &output.address.to_vec(),
                                validation_report,
                                decoded_metadata,
                                &witness,
                                txn_idx,
                            );
                        }
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction outputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    // Handle positive payment keys (reference to tx input)
                    let inputs = &tx.transaction_body.inputs;
                    let index = match decremented_index(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to get index of payment key: {e}"),
                                validation_report,
                                decoded_metadata,
                            );
                            return None;
                        },
                    };
                    if inputs.get(index).is_none() {
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction inputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    return Some(true);
                },
                MultiEraTxWithRawAuxiliary::Babbage(tx) => {
                    // Negative indicates reference to tx output
                    if payment_key < 0 {
                        let index = match decremented_index(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to get index of payment key: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };
                        let outputs = tx.transaction_body.outputs.clone();
                        let witness = match TxWitness::new(&[txn.clone()]) {
                            Ok(witnesses) => witnesses,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to create TxWitness: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };
                        if let Some(output) = outputs.get(index) {
                            match output {
                                pallas::ledger::primitives::babbage::PseudoTransactionOutput::Legacy(o) => {
                                    return self.validate_payment_output_key_helper(&o.address.to_vec(), validation_report, decoded_metadata, &witness, txn_idx);
                                }
                                ,
                                pallas::ledger::primitives::babbage::PseudoTransactionOutput::PostAlonzo(o) => {
                                    return self.validate_payment_output_key_helper(&o.address.to_vec(), validation_report, decoded_metadata, &witness, txn_idx)
                                }
                                ,
                            };
                        }
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction outputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    // Positive indicates reference to tx input
                    let inputs = &tx.transaction_body.inputs;
                    let index = match decremented_index(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to get index of payment key: {e}"),
                                validation_report,
                                decoded_metadata,
                            );
                            return None;
                        },
                    };
                    if inputs.get(index).is_none() {
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction inputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    return Some(true);
                },
                MultiEraTxWithRawAuxiliary::Conway(tx) => {
                    // Negative indicates reference to tx output
                    if payment_key < 0 {
                        let index = match decremented_index(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to get index of payment key: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };
                        let outputs = tx.transaction_body.outputs.clone();
                        let witness = match TxWitness::new(&[txn.clone()]) {
                            Ok(witnesses) => witnesses,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to create TxWitness: {e}"),
                                    validation_report,
                                    decoded_metadata,
                                );
                                return None;
                            },
                        };

                        if let Some(output) = outputs.get(index) {
                            match output {
                                 pallas::ledger::primitives::conway::PseudoTransactionOutput::Legacy(o) => {
                                     return self.validate_payment_output_key_helper(&o.address.to_vec(), validation_report, decoded_metadata, &witness, txn_idx);
                                 },
                                 pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(o) => {
                                     return self.validate_payment_output_key_helper(&o.address.to_vec(), validation_report, decoded_metadata, &witness, txn_idx);
                                 },
                             };
                        }
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction outputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    // Positive indicates reference to tx input
                    let inputs = &tx.transaction_body.inputs;
                    let index = match decremented_index(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to get index of payment key: {e}"),
                                validation_report,
                                decoded_metadata,
                            );
                            return None;
                        },
                    };
                    // Check whether the index exists in transaction inputs
                    if inputs.get(index).is_none() {
                        self.validation_failure(
                            "Role payment key reference index is not found in transaction inputs",
                            validation_report,
                            decoded_metadata,
                        );
                        return None;
                    }
                    return Some(true);
                },
                _ => {
                    self.validation_failure(
                        "Unsupported transaction era for payment key validation",
                        validation_report,
                        decoded_metadata,
                    );
                    return None;
                },
            }
        }
        Some(false)
    }

    /// Helper function for validating payment output key.
    fn validate_payment_output_key_helper(
        &self, output_address: &[u8], validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, witness: &TxWitness, txn_idx: usize,
    ) -> Option<bool> {
        let idx = match u16::try_from(txn_idx) {
            Ok(value) => value,
            Err(e) => {
                self.validation_failure(
                    &format!("Transaction index conversion failed: {e}"),
                    validation_report,
                    decoded_metadata,
                );
                return None;
            },
        };
        // Extract the key hash from the output address
        if let Some(key) = extract_key_hash(output_address) {
            // Compare the key hash and return the result
            return Some(compare_key_hash(&[key], witness, idx).is_ok());
        }
        self.validation_failure(
            "Failed to extract payment key hash from address",
            validation_report,
            decoded_metadata,
        );
        None
    }
}

#[cfg(test)]
mod tests {

    use dashmap::DashMap;

    use super::*;
    fn conway_1() -> Vec<u8> {
        hex::decode(include_str!(
            "../../../test_data/conway_tx_rbac/conway_1.block"
        ))
        .expect("Failed to decode hex block.")
    }

    fn conway_2() -> Vec<u8> {
        hex::decode(include_str!(
            "../../../test_data/conway_tx_rbac/conway_2.block"
        ))
        .expect("Failed to decode hex block.")
    }

    fn conway_3() -> Vec<u8> {
        hex::decode(include_str!(
            "../../../test_data/conway_tx_rbac/conway_3.block"
        ))
        .expect("Failed to decode hex block.")
    }

    fn cip_509_aux_data(tx: &pallas::ledger::traverse::MultiEraTxWithRawAuxiliary<'_>) -> Vec<u8> {
        let raw_auxiliary_data = tx
            .as_conway()
            .unwrap()
            .clone()
            .auxiliary_data
            .map(|aux| aux.raw_cbor());

        let raw_cbor_data = match raw_auxiliary_data {
            pallas::codec::utils::Nullable::Some(data) => Ok(data),
            _ => Err("Auxiliary data not found"),
        };

        let auxiliary_data = RawAuxData::new(raw_cbor_data.expect("Failed to get raw cbor data"));
        auxiliary_data
            .get_metadata(LABEL)
            .expect("Failed to get metadata")
            .to_vec()
    }

    #[test]
    fn test_decode_cip509() {
        // This data is from conway_1.block
        let cip_509 = "a50050ca7a1457ef9f4c7f9c747f8c4a4cfa6c0150226d126819472b7afad7d0b8c7b89aa20258204d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c20b9458401b03060066006fd5b67002167882eac0b5f2b11da40788a39bfa0324c494f7003a6b4c1c4bac378e322cb280230a4002f5b2754e863806f7e524afc99996aa28584032f02b600cbf04c6a09e05100880a09ee59b6627dc78d68175469b8c5b1fac141a6da5c6c2ea446597b6f0b6efea00a04ac0c1756455589908a5e089ba604a1258405917d6ee2b2535959d806c00eb2958929ababb40d681b5245751538e915d3d90f561ddcaa9aaa9cd78a30882a22a99c742c4f7610b43750a0d6651e8640a8d4c58402167427cfa933d6430c026640888210cd0c4e93e7015100300dcaef47b9c155ea4ccb27773c27f5d6a44fbf98065a14e5f0eca530e57082a971cbf22fa9065585840ae72e2a061eb558d3fd7727e87a8f07b5faf0d3cedf8d99ab6e0c845f5dd3ce78d31d7365c523b5a4dfe5d35bfafaefb2f60dd7473cbe8d6aa6bf557b1fbdf775840bf96bcd3ffdbfc7d20b65be7f5c7dba1cf635e3b449bb644bdfc73e0e49a5db73edddc7ed331220ba732f62f3aee8503f5c6f9bd5f7fedb37dc6580196052e50584027fdd7e8bfe9146561ad1ebc79ecef0ee1df7081cf9cd1fd929569ef3d55972d5b7ff882ce2213f789fc08787164f14aa86d55e98e332b220a07fa464aaa7c335840ce4bcfb268ed577f72e87fdea4442107bf2da93fe05121d5befa7ae5aecc5f3f9c732e82108003166380198c0146b0e214114a31d7c62b0ec18afd5834034c2b58402b2c515b350d8980a16932071b6d8d125ea1eb53dc28a8aee1787a1670b9e8c4c8cb00c726f3515a39ca1689f870295752820a64721e05e1a234710583416316584031d80291ac9a2b66a04cba844b85f9928a4a04a9928b2805124a25b3aaa4422e45e5d422a9b88a028ba4a5123ac244b8b472164b86085ac21357c3aae7696be25840f1104878009b03813d9e6c53255722402090206058a009d2b808aff772fb712d75f1dea09507fd83838e045dd9ce3eb59e4554f5ed02b8aeb60700f4b39dd9fe584064e1d5a137de0fa4c6cccfd71f831bee372756d72990b357a44e2f9eaf3854db65379db466cfcb55517ba71550acade564f4b7efd1fd95fa57228cee6fa9ae3458405ce1ae79b77f7cd5bdecfcb800fbdb7eaf720eae5995176d94a07c326c71aaf5e6f8439e577edb2d1ed64959324b5a7476e9159bf37bdf226edb747787b79b9e5840bc6ab5b84714eefa4a8c2df4aba37a36757d8b39dd79ec41b4a2f3ee96eabdc0e1f65b37264bdbfdf79eebbc820a7deab4e39f7e1cbf6610402fd8fb55fbef3d584038226e4d37c42970c830184b2e1c5026eadb9677ae8f6d300975ca6ceec5c8920382e827c1f636f7dd9f8d492737f4520a944bfeebba5ca2d5efa80ad453a43f584004c357ecccfc4dab75ce560b0300db9092ced52625d0c8df6fc89da9a45b6dc9c2461f21e6ee7b7afd877fbd8c1a1fa7ff38fa506e14749ebb68e24571c6220c584004208c284d628c2148b252f91b8b50014b080b040554095b52ca862bb974218222d412112ae5d2584c54584ae157f22b183cb4ba9c5fc42ba6894ad074ffe0875840c69ee921211d0ce4cd0f89b7e708163b3ab9286fe26a8c68ed85930cabc5dbfed7f9681c535dbdbfeb56f7a2b32d1f43de1dbcc934676edefacb3df7c1210067584064a1b8d94448b7f22a77dc736edb12f7c2c52b2eb8d4a80b78147d89f9a3a0659c03e10bbb336e391b3961f1afbfa08af3de2a817fceddea0cb57f438b0f8947581e9782ee92e890df65636d835d2d465cc5521c0ec05470e002800015eecf5818635840e0427f23196c17cf13f030595335343030c11d914bc7a84b56af7040930af4110fd4ca29b0bc0e83789adb8668ea2ef28c1dd10dc1fd35ea6ae8c06ee769540d";
        let binding = hex::decode(cip_509).unwrap();
        let mut decoder = Decoder::new(binding.as_slice());
        let decoded_cip509 = Cip509::decode(&mut decoder, &mut ()).unwrap();

        let purpose: [u8; 16] = hex::decode("ca7a1457ef9f4c7f9c747f8c4a4cfa6c")
            .unwrap()
            .try_into()
            .unwrap();
        let txn_inputs_hash: [u8; 16] = hex::decode("226d126819472b7afad7d0b8c7b89aa2")
            .unwrap()
            .try_into()
            .unwrap();
        let prv_tx_id: [u8; 32] =
            hex::decode("4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2")
                .unwrap()
                .try_into()
                .unwrap();
        let validation_signature = hex::decode("e0427f23196c17cf13f030595335343030c11d914bc7a84b56af7040930af4110fd4ca29b0bc0e83789adb8668ea2ef28c1dd10dc1fd35ea6ae8c06ee769540d").unwrap();

        assert_eq!(decoded_cip509.purpose, purpose);
        assert_eq!(decoded_cip509.txn_inputs_hash, txn_inputs_hash);
        assert_eq!(decoded_cip509.prv_tx_id, Some(prv_tx_id));
        assert_eq!(decoded_cip509.validation_signature, validation_signature);
    }

    #[test]
    fn test_validate_txn_inputs_hash() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");
        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_txn_inputs_hash(tx, &mut validation_report, &decoded_metadata)
            .unwrap());
    }

    #[test]
    fn test_validate_aux() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_aux(tx, &mut validation_report, &decoded_metadata)
            .unwrap());
    }

    #[test]
    fn test_validate_public_key_success() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_stake_public_key(tx, &mut validation_report, &decoded_metadata, 0)
            .unwrap());
    }

    #[test]
    fn test_validate_payment_key_success_positive_ref() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");

        if let Some(role_set) = &cip509.x509_chunks.0.role_set {
            for role in role_set {
                if role.role_number == 0 {
                    assert!(cip509
                        .validate_payment_key(
                            tx,
                            &mut validation_report,
                            &decoded_metadata,
                            0,
                            role
                        )
                        .unwrap());
                }
            }
        }
    }

    #[test]
    fn test_validate_payment_key_success_negative_ref() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_3();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .first()
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");

        if let Some(role_set) = &cip509.x509_chunks.0.role_set {
            for role in role_set {
                if role.role_number == 0 {
                    println!(
                        "{:?}",
                        cip509.validate_payment_key(
                            tx,
                            &mut validation_report,
                            &decoded_metadata,
                            0,
                            role
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn test_validate_public_key_fail() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_2();
        let multi_era_block =
            pallas::ledger::traverse::MultiEraBlockWithRawAuxiliary::decode(&conway_block_data)
                .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(!cip509
            .validate_stake_public_key(tx, &mut validation_report, &decoded_metadata, 0)
            .unwrap());
    }
}
