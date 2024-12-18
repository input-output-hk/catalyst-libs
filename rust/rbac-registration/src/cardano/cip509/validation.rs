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
//! * Role signing key validation for role 0 where the signing keys should only be the
//!   certificates
//!
//!  See:
//! * <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! * <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>
//!
//! Note: This CIP509 is still under development and is subject to change.

use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::{addresses::Address, traverse::MultiEraTx},
};

use super::{
    blake2b_128, blake2b_256, decremented_index,
    rbac::role_data::{LocalRefInt, RoleData},
    utils::cip19::{compare_key_hash, extract_key_hash},
    Cip509, TxInputHash, TxWitness,
};
use crate::{cardano::cip509::utils::Cip0134UriList, utils::general::zero_out_last_n_bytes};

/// Context-specific primitive type with tag number 6 (`raw_tag` 134) for
/// uniform resource identifier (URI) in the subject alternative name extension.
/// Following the ASN.1
/// <https://www.oss.com/asn1/resources/asn1-made-simple/asn1-quick-reference/asn1-tags.html>
/// the tag is derive from
/// | Class   (2 bit)    | P/C  (1 bit)   | Tag Number (5 bit) |
/// |`CONTEXT_SPECIFIC`  | `PRIMITIVE`   `|      6`            |
/// |`10`                | `0`           `|      00110`        |
/// Result in 0x86 or 134 in decimal.
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
        Some(TxInputHash::from(inputs_hash) == cip509.txn_inputs_hash)
    } else {
        validation_report.push(format!("{function_name}, Unsupported transaction era for"));
        None
    }
}

// ------------------------ Validate Stake Public Key ------------------------

/// Validate the stake public key in the certificate with witness set in transaction.
pub(crate) fn validate_stake_public_key(
    cip509: &Cip509, txn: &MultiEraTx, validation_report: &mut Vec<String>,
) -> Option<bool> {
    let function_name = "Validate Stake Public Key";

    if !matches!(txn, MultiEraTx::Conway(_)) {
        validation_report.push(format!("{function_name}, Unsupported transaction era"));
        return None;
    }
    let addresses = Cip0134UriList::new(cip509);

    // Create TxWitness
    // Note that TxWitness designs to work with multiple transactions
    let witnesses = match TxWitness::new(&[txn.clone()]) {
        Ok(witnesses) => witnesses,
        Err(e) => {
            validation_report.push(format!("{function_name}, Failed to create TxWitness: {e}"));
            return None;
        },
    };

    // TODO: Update compare_key_hash to accept Cip0134UriList?
    let pk_addrs: Vec<_> = addresses
        .iter()
        .filter_map(|a| {
            if let Address::Stake(a) = a.address() {
                Some(a.payload().as_hash().to_vec())
            } else {
                None
            }
        })
        .collect();

    Some(
        // Set transaction index to 0 because the list of transaction is manually constructed
        // for TxWitness -> &[txn.clone()], so we can assume that the witness contains only
        // the witness within this transaction.
        compare_key_hash(&pk_addrs, &witnesses, 0)
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
/// Also return the pre-computed hash where the validation signature (99) set to
pub(crate) fn validate_aux(
    txn: &MultiEraTx, validation_report: &mut Vec<String>,
) -> Option<(bool, Vec<u8>)> {
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
/// Also compute The pre-computed hash.
fn validate_aux_helper(
    original_aux: &[u8], aux_data_hash: &Bytes, validation_report: &mut Vec<String>,
) -> Option<(bool, Vec<u8>)> {
    let mut vec_aux = original_aux.to_vec();

    // Pre-computed aux with the last 64 bytes set to zero
    zero_out_last_n_bytes(&mut vec_aux, 64);

    // Compare the hash
    match blake2b_256(original_aux) {
        Ok(original_hash) => {
            return Some((aux_data_hash.as_ref() == original_hash, vec_aux));
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
    txn: &MultiEraTx, role_data: &RoleData, validation_report: &mut Vec<String>,
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
                            );
                        },
                        pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(
                            o,
                        ) => {
                            return validate_payment_output_key_helper(
                                &o.address.to_vec(),
                                validation_report,
                                &witness,
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
    output_address: &[u8], validation_report: &mut Vec<String>, witness: &TxWitness,
) -> Option<bool> {
    // Extract the key hash from the output address
    if let Some(key) = extract_key_hash(output_address) {
        // Compare the key hash and return the result
        // Set transaction index to 0 because the list of transaction is manually constructed
        // for TxWitness -> &[txn.clone()], so we can assume that the witness contains only
        // the witness within this transaction.
        return Some(compare_key_hash(&[key], witness, 0).is_ok());
    }
    validation_report.push("Failed to extract payment key hash from address".to_string());
    None
}

// ------------------------ Validate role signing key ------------------------

/// Validate role singing key for role 0.
/// Must reference certificate not the public key
pub(crate) fn validate_role_singing_key(
    role_data: &RoleData, validation_report: &mut Vec<String>,
) -> bool {
    let function_name = "Validate Role Signing Key";

    // If signing key exist, it should not contain public key
    if let Some(local_ref) = &role_data.role_signing_key {
        if local_ref.local_ref == LocalRefInt::PubKeys {
            validation_report.push(format!(
                "{function_name}, Role signing key should reference certificate, not public key",
            ));
            println!("ja");
            return false;
        }
    }

    true
}

// ------------------------ Tests ------------------------

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

        validate_aux(tx, &mut validation_report);
        assert!(validate_aux(tx, &mut validation_report).unwrap().0);
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
        assert!(validate_stake_public_key(&cip509, tx, &mut validation_report).unwrap());
    }

    #[test]
    fn test_validate_payment_key_success_negative_ref() {
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
                    assert!(validate_payment_key(tx, role, &mut validation_report,).unwrap());
                }
            }
        }
    }

    #[test]
    fn test_role_0_signing_key() {
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
                    assert!(validate_role_singing_key(role, &mut validation_report,));
                }
            }
        }
    }

    #[test]
    fn test_validate_payment_key_success_positive_ref() {
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
                    assert!(validate_payment_key(tx, role, &mut validation_report,).unwrap());
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
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .first()
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        assert!(!validate_stake_public_key(&cip509, tx, &mut validation_report).unwrap());
    }
}
