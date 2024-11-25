//! Basic validation for CIP-0509
//! The validation include the following:
//! * Hashing the transaction inputs within the transaction should match the
//!   txn-inputs-hash in CIP-0509 data.
//! * Auxiliary data hash within the transaction should match the hash of the auxiliary
//!   data itself.
//! * Public key validation for role 0 where public key extracted from x509 and c509
//!   subject alternative name should match one of the witness in witness set within the
//!   transaction.
//! * Payment key reference validation for role 0 where the reference should be either
//!     1. Negative index reference - reference to transaction output in transaction:
//!        should match some of the key within witness set.
//!     2. Positive index reference - reference to the transaction input in transaction:
//!        only check whether the index exist within the transaction inputs.
//!
//!  See:
//! * <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! * <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>
//!
//! Note: This CIP509 is still under development and is subject to change.

use c509_certificate::{general_names::general_name::GeneralNameValue, C509ExtensionType};
use der_parser::der::parse_der_sequence;
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::traverse::MultiEraTx,
};
use x509_cert::der::{oid::db::rfc5912::ID_CE_SUBJECT_ALT_NAME, Decode};

use super::{
    blake2b_128, blake2b_256, decode_utf8, decremented_index,
    rbac::{
        certs::{C509Cert, X509DerCert},
        role_data::RoleData,
    },
    utils::cip19::{compare_key_hash, extract_cip19_hash, extract_key_hash},
    Cip509, TxInputHash, TxWitness,
};

/// Context-specific primitive type with tag number 6 (`raw_tag` 134) for
/// uniform resource identifier (URI) in the subject alternative name extension.
pub(crate) const URI: u8 = 134;

// ------------------------ Validate Txn Inputs Hash ------------------------

/// Transaction inputs hash validation.
/// CIP509 `txn_inputs_hash` must match the hash of the transaction inputs within the
/// body.
pub(crate) fn validate_txn_inputs_hash(
    cip509: &Cip509, txn: &MultiEraTx, validation_report: &mut Vec<String>,
) -> Option<bool> {
    let function_name = "Validate Transaction Inputs Hash";
    let mut buffer = Vec::new();
    let mut e = Encoder::new(&mut buffer);
    // CIP-0509 should only be in conway era
    if let MultiEraTx::Conway(tx) = txn {
        let inputs = tx.transaction_body.inputs.clone();
        if let Err(e) = e.array(inputs.len() as u64) {
            validation_report.push(format!(
                "{function_name}, Failed to encode array of transaction input: {e}"
            ));
            return None;
        }
        for input in &inputs {
            match input.encode(&mut e, &mut ()) {
                Ok(()) => {},
                Err(e) => {
                    validation_report.push(format!(
                        "{function_name}, Failed to encode transaction input {e}"
                    ));
                    return None;
                },
            }
        }
        // Hash the transaction inputs
        let inputs_hash = match blake2b_128(&buffer) {
            Ok(hash) => hash,
            Err(e) => {
                validation_report.push(format!(
                    "{function_name}, Failed to hash transaction inputs {e}"
                ));
                return None;
            },
        };
        Some(TxInputHash(inputs_hash) == cip509.txn_inputs_hash)
    } else {
        validation_report.push(format!("{function_name}, Unsupported transaction era for"));
        None
    }
}

// ------------------------ Validate Stake Public Key ------------------------

/// Validate the stake public key in the certificate with witness set in transaction.
#[allow(clippy::too_many_lines)]
pub(crate) fn validate_stake_public_key(
    cip509: &Cip509, txn: &MultiEraTx, txn_idx: usize, validation_report: &mut Vec<String>,
) -> Option<bool> {
    let function_name = "Validate Stake Public Key";
    let mut pk_addrs = Vec::new();

    // CIP-0509 should only be in conway era
    if let MultiEraTx::Conway(_) = txn {
        // X509 certificate
        if let Some(x509_certs) = &cip509.x509_chunks.0.x509_certs {
            for x509_cert in x509_certs {
                match x509_cert {
                    X509DerCert::X509Cert(cert) => {
                        // Attempt to decode the DER certificate
                        let der_cert = match x509_cert::Certificate::from_der(cert) {
                            Ok(cert) => cert,
                            Err(e) => {
                                validation_report.push(format!(
                                    "{function_name}, Failed to decode x509 certificate DER: {e}"
                                ));
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
                                        // Check for context-specific primitive type with tag
                                        // number
                                        // 6 (raw_tag 134)
                                        if data.header.raw_tag() == Some(&[URI]) {
                                            match data.content.as_slice() {
                                                Ok(content) => {
                                                    // Decode the UTF-8 string
                                                    let addr: String = match decode_utf8(content) {
                                                        Ok(addr) => addr,
                                                        Err(e) => {
                                                            validation_report.push(format!(
                                                                    "{function_name}, Failed to decode UTF-8 string for context-specific primitive type with raw tag 134: {e}",
                                                                ),
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
                                                    validation_report.push(
                                                        format!("{function_name}, Failed to process content for context-specific primitive type with raw tag 134: {e}"));
                                                    return None;
                                                },
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    validation_report.push(
                                        format!(
                                            "{function_name}, Failed to parse DER sequence for Subject Alternative Name extension: {e}"
                                        )
                                    );
                                    return None;
                                },
                            }
                        }
                    },
                    _ => continue,
                }
            }
        }
        // C509 Certificate
        if let Some(c509_certs) = &cip509.x509_chunks.0.c509_certs {
            for c509_cert in c509_certs {
                match c509_cert {
                    C509Cert::C509CertInMetadatumReference(_) => {
                        validation_report.push(format!(
                            "{function_name}, C509 metadatum reference is currently not supported"
                        ));
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
                                                                        validation_report.push(
                                                                            format!("{function_name}, Failed to get the value of subject alternative name"),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    },
                                                    c509_certificate::extensions::alt_name::GeneralNamesOrText::Text(_) => {
                                                        validation_report.push(
                                                            format!("{function_name}, Failed to find C509 general names in subject alternative name"),
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {
                                                validation_report.push(
                                                    format!("{function_name}, Failed to get C509 subject alternative name")
                                                );
                                            }
                                        }
                            }
                        }
                    },
                    _ => continue,
                }
            }
        }
    } else {
        validation_report.push(format!("{function_name}, Unsupported transaction era"));
        return None;
    }

    // Create TxWitness
    let witnesses = match TxWitness::new(&[txn.clone()]) {
        Ok(witnesses) => witnesses,
        Err(e) => {
            validation_report.push(format!("{function_name}, Failed to create TxWitness: {e}"));
            return None;
        },
    };

    let index = match u16::try_from(txn_idx) {
        Ok(value) => value,
        Err(e) => {
            validation_report.push(format!(
                "{function_name}, Failed to convert transaction index to usize: {e}"
            ));
            return None;
        },
    };
    Some(
        compare_key_hash(&pk_addrs, &witnesses, index)
            .map_err(|e| {
                validation_report.push(format!(
                    "{function_name}, Failed to compare public keys with witnesses: {e}"
                ));
            })
            .is_ok(),
    )
}

// ------------------------ Validate Aux ------------------------

/// Validate the auxiliary data with the auxiliary data hash in the transaction body.
pub(crate) fn validate_aux(txn: &MultiEraTx, validation_report: &mut Vec<String>) -> Option<bool> {
    let function_name = "Validate Aux";

    // CIP-0509 should only be in conway era
    if let MultiEraTx::Conway(tx) = txn {
        if let pallas::codec::utils::Nullable::Some(a) = &tx.auxiliary_data {
            let original_aux = a.raw_cbor();
            let aux_data_hash = tx
                .transaction_body
                .auxiliary_data_hash
                .as_ref()
                .or_else(|| {
                    validation_report.push(format!(
                        "{function_name}, Auxiliary data hash not found in transaction"
                    ));
                    None
                })?;
            validate_aux_helper(original_aux, aux_data_hash, validation_report)
        } else {
            validation_report.push(format!(
                "{function_name}, Auxiliary data not found in transaction"
            ));
            None
        }
    } else {
        validation_report.push(format!("{function_name}, Unsupported transaction era"));
        None
    }
}

/// Helper function for auxiliary data validation.
fn validate_aux_helper(
    original_aux: &[u8], aux_data_hash: &Bytes, validation_report: &mut Vec<String>,
) -> Option<bool> {
    // Compare the hash
    match blake2b_256(original_aux) {
        Ok(original_hash) => {
            return Some(aux_data_hash.as_ref() == original_hash);
        },
        Err(e) => {
            validation_report.push(format!("Cannot hash auxiliary data {e}"));
            None
        },
    }
}

// ------------------------ Validate Payment Key ------------------------

/// Validate the payment key reference.
/// Negative ref is for transaction output.
/// Positive ref is for transaction input.
pub(crate) fn validate_payment_key(
    txn: &MultiEraTx, txn_idx: usize, role_data: &RoleData, validation_report: &mut Vec<String>,
) -> Option<bool> {
    let function_name = "Validate Payment Key";

    if let Some(payment_key) = role_data.payment_key {
        if payment_key == 0 {
            validation_report.push(format!(
                "{function_name}, Invalid payment reference key, 0 is not allowed"
            ));
            return None;
        }
        // CIP-0509 should only be in conway era
        if let MultiEraTx::Conway(tx) = txn {
            // Negative indicates reference to tx output
            if payment_key < 0 {
                let index = match decremented_index(payment_key.abs()) {
                    Ok(value) => value,
                    Err(e) => {
                        validation_report.push(format!(
                            "{function_name}, Failed to get index of payment key: {e}"
                        ));
                        return None;
                    },
                };
                let outputs = tx.transaction_body.outputs.clone();
                let witness = match TxWitness::new(&[txn.clone()]) {
                    Ok(witnesses) => witnesses,
                    Err(e) => {
                        validation_report
                            .push(format!("{function_name}, Failed to create TxWitness: {e}"));
                        return None;
                    },
                };

                if let Some(output) = outputs.get(index) {
                    match output {
                        pallas::ledger::primitives::conway::PseudoTransactionOutput::Legacy(o) => {
                            return validate_payment_output_key_helper(
                                &o.address.to_vec(),
                                validation_report,
                                &witness,
                                txn_idx,
                            );
                        },
                        pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(
                            o,
                        ) => {
                            return validate_payment_output_key_helper(
                                &o.address.to_vec(),
                                validation_report,
                                &witness,
                                txn_idx,
                            );
                        },
                    };
                }
                validation_report.push(
                    format!("{function_name}, Role payment key reference index is not found in transaction outputs")
                );
                return None;
            }
            // Positive indicates reference to tx input
            let inputs = &tx.transaction_body.inputs;
            let index = match decremented_index(payment_key) {
                Ok(value) => value,
                Err(e) => {
                    validation_report.push(format!(
                        "{function_name}, Failed to get index of payment key: {e}"
                    ));
                    return None;
                },
            };
            // Check whether the index exists in transaction inputs
            if inputs.get(index).is_none() {
                validation_report.push(
                    format!("{function_name}, Role payment key reference index is not found in transaction inputs")
                );
                return None;
            }
            Some(true)
        } else {
            validation_report.push(format!(
                "{function_name}, Unsupported transaction era for stake payment key validation"
            ));
            None
        }
    } else {
        Some(false)
    }
}

/// Helper function for validating payment output key.
fn validate_payment_output_key_helper(
    output_address: &[u8], validation_report: &mut Vec<String>, witness: &TxWitness, txn_idx: usize,
) -> Option<bool> {
    let idx = match u16::try_from(txn_idx) {
        Ok(value) => value,
        Err(e) => {
            validation_report.push(format!("Transaction index conversion failed: {e}"));
            return None;
        },
    };
    // Extract the key hash from the output address
    if let Some(key) = extract_key_hash(output_address) {
        // Compare the key hash and return the result
        return Some(compare_key_hash(&[key], witness, idx).is_ok());
    }
    validation_report.push("Failed to extract payment key hash from address".to_string());
    None
}

#[cfg(test)]
mod tests {

    use minicbor::{Decode, Decoder};

    use super::*;
    use crate::cardano::transaction::raw_aux_data::RawAuxData;

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
            .get_metadata(509)
            .expect("Failed to get metadata")
            .to_vec()
    }

    fn conway_1() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_1.block"))
            .expect("Failed to decode hex block.")
    }

    fn conway_2() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_2.block"))
            .expect("Failed to decode hex block.")
    }

    fn conway_3() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_3.block"))
            .expect("Failed to decode hex block.")
    }

    #[test]
    fn test_validate_txn_inputs_hash() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");
        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(validate_txn_inputs_hash(&cip509, tx, &mut validation_report).unwrap());
    }

    #[test]
    fn test_validate_aux() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        // let aux_data = cip_509_aux_data(tx);

        // let mut decoder = Decoder::new(aux_data.as_slice());
        // let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        validate_aux(tx, &mut validation_report);
        assert!(validate_aux(tx, &mut validation_report).unwrap());
    }

    #[test]
    fn test_validate_public_key_success() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(validate_stake_public_key(&cip509, tx, 0, &mut validation_report).unwrap());
    }

    #[test]
    fn test_validate_payment_key_success_positive_ref() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_1();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
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
                    assert!(validate_payment_key(tx, 0, role, &mut validation_report,).unwrap());
                }
            }
        }
    }

    #[test]
    fn test_validate_payment_key_success_negative_ref() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_3();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
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
                        validate_payment_key(tx, 0, role, &mut validation_report,)
                    );
                }
            }
        }
    }

    #[test]
    fn test_validate_public_key_fail() {
        let mut validation_report = Vec::new();
        let conway_block_data = conway_2();
        let multi_era_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data)
            .expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(!validate_stake_public_key(&cip509, tx, 0, &mut validation_report).unwrap());
    }
}
