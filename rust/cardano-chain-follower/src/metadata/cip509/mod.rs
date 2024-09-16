//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix
use c509_certificate::general_names::general_name::GeneralNameValue;
use decode_helper::{decode_bytes, decode_map_len, decode_u8};
use der_parser::{asn1_rs::oid, der::parse_der_sequence, Oid};
use rbac::{certs::C509Cert, role_data::RoleData};

mod decode_helper;
mod rbac;
mod utils;
use utils::{compare_key_hash, extract_cip19_hash, extract_key_hash, zero_out_last_n_bytes};
use x509_cert::{der::Decode as _, ext::pkix::ID_CE_SUBJECT_ALT_NAME};
mod x509_chunks;

use std::sync::Arc;

use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::traverse::MultiEraTx,
};
use strum::FromRepr;
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

/// Subject Alternative Name OID
pub(crate) const SUBJECT_ALT_NAME_OID: Oid = oid!(2.5.29 .17);

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
                decode_u8(d, "CIP509")?;
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
        decoded_metadata: &DecodedMetadata, txn: &MultiEraTx, raw_aux_data: &RawAuxData,
        txn_idx: usize,
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
        &self, txn: &MultiEraTx, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) -> Option<bool> {
        let mut buffer = Vec::new();
        let mut e = Encoder::new(&mut buffer);
        match txn {
            MultiEraTx::AlonzoCompatible(tx, _) => {
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
            MultiEraTx::Babbage(tx) => {
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
            MultiEraTx::Conway(tx) => {
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
        &mut self, txn: &MultiEraTx, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) -> Option<bool> {
        match txn {
            MultiEraTx::AlonzoCompatible(tx, _) => {
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
            MultiEraTx::Babbage(tx) => {
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
            MultiEraTx::Conway(tx) => {
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
        &self, txn: &MultiEraTx, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, txn_idx: usize,
    ) -> Option<bool> {
        let mut pk_addrs = Vec::new();
        match txn {
            MultiEraTx::AlonzoCompatible(..) | MultiEraTx::Babbage(_) | MultiEraTx::Conway(_) => {
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
                                for exts in c509.get_tbs_cert().get_extensions().get_inner() {
                                    if exts.get_registered_oid().get_c509_oid().get_oid()
                                        == SUBJECT_ALT_NAME_OID
                                    {
                                        match exts.get_value() {
                                            c509_certificate::extensions::extension::ExtensionValue::AlternativeName(alt_name) => {
                                                match alt_name.get_inner() {
                                                    c509_certificate::extensions::alt_name::GeneralNamesOrText::GeneralNames(gn) => {
                                                        for name in gn.get_inner() {
                                                            if name.get_gn_type() == &c509_certificate::general_names::general_name::GeneralNameTypeRegistry::UniformResourceIdentifier {
                                                                match name.get_gn_value() {
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
        &self, txn: &MultiEraTx, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, txn_idx: usize, role_data: &RoleData,
    ) -> Option<bool> {
        if let Some(payment_key) = role_data.payment_key {
            match txn {
                MultiEraTx::AlonzoCompatible(tx, _) => {
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
                        let index = match usize::try_from(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to convert payment_key to usize: {e}"),
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
                    let index = match usize::try_from(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to convert payment_key to isize: {e}"),
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
                MultiEraTx::Babbage(tx) => {
                    // Negative indicates reference to tx output
                    if payment_key < 0 {
                        let index = match usize::try_from(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to convert payment_key to usize: {e}"),
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
                    let index = match usize::try_from(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to convert payment_key to isize: {e}"),
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
                MultiEraTx::Conway(tx) => {
                    // Negative indicates reference to tx output
                    if payment_key < 0 {
                        let index = match usize::try_from(payment_key.abs()) {
                            Ok(value) => value,
                            Err(e) => {
                                self.validation_failure(
                                    &format!("Failed to convert payment_key to usize: {e}"),
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
                    let index = match usize::try_from(payment_key) {
                        Ok(value) => value,
                        Err(e) => {
                            self.validation_failure(
                                &format!("Failed to convert payment_key to isize: {e}"),
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

    fn cip_509_aux_data(tx: &MultiEraTx<'_>) -> Vec<u8> {
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
        let cip_509 = "a50050ca7a1457ef9f4c7f9c747f8c4a4cfa6c01504dd9d3b2ef173daf8612819857721d4b0258204d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c20b8c58401b3d030866084fcb259de07496d3197e913a39fd628a3db0a4ed6839261a00c51cb0a5b9c16194064132ace375ea23c75c60659400cba304d0d689c00086195d5840ff28714da02c35e7295815ba58b77f227e576fa254c464e2f9c6f9dfa900a0208250033c054a468c38e08819601d073c034a4727a524ff39995477443c1fca235840839c927599b253887f50487c1caf757c0aaf79bc3fcacd42252b8f2ae1f1a8b282929ca22bb5c2885cc23a66005c0cc1ca20142b82310c3a137d44c1943e40995840a7a7ce5c3475b5887a3765ede2ff3b7bfea90f255e2edf37fd44e27f26b8e6cf408aef4b20bebf7257b3dabc7eda65fff4ed278b50219f0a52367ff5b80e46b758403875f55a394d17a5d9a6b1a1deff5b2206e9e9734e9fbefa6a1cdfeb7a104546dfb6e46c46feaeb65a7f4648c276e29e87b27bc053bffef79359300220d0c3875840f2a05cc4880317358e19c758fd9ab9917551ce3987af2e35d73b6958a0f5732784621b0c92f68a93537f16f48445424890f955d7a597c13c2eb54a82b39f0307584097507df5fef916fabb6dafdfb516fb9184783e2cb4e89d048a6c1e5c04818bdb76ffb5cbef1fbe452658d904cd152ee72a3bfc6efe1199fb3b51f1979629cd4e5840fdb7df511723d4cead3d2b2eb9c1f18cbbfcf9f5cc8eac46dc03cd55fcac3303c391437f50400923e65c02e981af5461b6867a47fb25ebe9b0fb4d9e41ec210e58404b9011000206414523c0990f9ee20b5d8a745393d3febaf6413a448b994f1567eb7945df7a0ab44afd55561e0190b376d411026c5d7a4a49a19e0bd3f5addd6c5840492fde46eee8d75b587286291dfeb6a78fdf59c1a6bfa2717b1f41dfa878756140ce7c77504b64b094b870ade78569566eec66369133af5aa8c8eab9f95e29df58409ec10be251547101b24c495c8ff4fa55378dbb4a5c6e89b18a12ac033343d61c3b7f5fba725b51536d92a5cbfaef9be6d24a3e5b3d75a1c0e29e42f523567fac4d0f8200811c822d2210b97f5708186358403b22c9d23b9e33092595b517442f4c73fbe11f2ec5bb7b3eb1ed060aeca73bfe750496dc8bdf459e9100c0013801dd1c6783d1703e18f738cf1b13561eaa1209";
        let binding = hex::decode(cip_509).unwrap();
        let mut decoder = Decoder::new(binding.as_slice());
        let decoded_cip509 = Cip509::decode(&mut decoder, &mut ()).unwrap();

        let purpose: [u8; 16] = hex::decode("ca7a1457ef9f4c7f9c747f8c4a4cfa6c")
            .unwrap()
            .try_into()
            .unwrap();
        let txn_inputs_hash: [u8; 16] = hex::decode("4dd9d3b2ef173daf8612819857721d4b")
            .unwrap()
            .try_into()
            .unwrap();
        let prv_tx_id: [u8; 32] =
            hex::decode("4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2")
                .unwrap()
                .try_into()
                .unwrap();
        let validation_signature = hex::decode("3b22c9d23b9e33092595b517442f4c73fbe11f2ec5bb7b3eb1ed060aeca73bfe750496dc8bdf459e9100c0013801dd1c6783d1703e18f738cf1b13561eaa1209").unwrap();

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
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[0].clone();
        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_txn_inputs_hash(&tx, &mut validation_report, &decoded_metadata)
            .unwrap());
    }

    #[test]
    fn test_validate_aux() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[0].clone();

        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_aux(&tx, &mut validation_report, &decoded_metadata)
            .unwrap());
    }

    #[test]
    fn test_validate_public_key_success() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[0].clone();

        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(cip509
            .validate_stake_public_key(&tx, &mut validation_report, &decoded_metadata, 0)
            .unwrap());
    }

    #[test]
    fn test_validate_payment_key_success_positive_ref() {
        let decoded_metadata = DecodedMetadata(DashMap::new());
        let mut validation_report = ValidationReport::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[0].clone();

        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");

        if let Some(role_set) = &cip509.x509_chunks.0.role_set {
            for role in role_set {
                if role.role_number == 0 {
                    assert!(cip509
                        .validate_payment_key(
                            &tx,
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
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Third transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[2].clone();

        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");

        if let Some(role_set) = &cip509.x509_chunks.0.role_set {
            for role in role_set {
                if role.role_number == 0 {
                    println!(
                        "{:?}",
                        cip509.validate_payment_key(
                            &tx,
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
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Fifth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions[4].clone();

        let aux_data = cip_509_aux_data(&tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(!cip509
            .validate_stake_public_key(&tx, &mut validation_report, &decoded_metadata, 0)
            .unwrap());
    }
}
