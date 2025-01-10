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

use catalyst_types::problem_report::ProblemReport;
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::{addresses::Address, primitives::conway, traverse::MultiEraTx},
};

use super::utils::cip19::{compare_key_hash, extract_key_hash};
use crate::{
    cardano::{
        cip509::{
            role_data::{LocalRefInt, RoleData},
            types::TxInputHash,
            Cip0134UriSet, Cip509,
        },
        transaction::witness::TxWitness,
    },
    utils::{
        general::decremented_index,
        hashing::{blake2b_128, blake2b_256},
    },
};

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

/// Checks that hashing transactions inputs produces the value equal to the given one.
pub fn validate_txn_inputs_hash(
    hash: &TxInputHash, transaction: &conway::MintedTx, report: &ProblemReport,
) {
    let context = "Cip509 transaction input hash validation";

    let mut buffer = Vec::new();
    let mut e = Encoder::new(&mut buffer);

    let inputs = &transaction.transaction_body.inputs;
    if let Err(e) = e.array(inputs.len() as u64) {
        report.other(
            &format!("Failed to encode array of transaction inputs: {e:?}"),
            context,
        );
        return;
    };
    for input in inputs {
        if let Err(e) = input.encode(&mut e, &mut ()) {
            report.other(
                &format!("Failed to encode transaction input ({input:?}): {e:?}"),
                context,
            );
            return;
        }
    }

    match blake2b_128(&buffer).map(TxInputHash::from) {
        Ok(h) if h == *hash => {
            // All good - the calculated hash is the same as in Cip509.
        },
        Ok(h) => {
            report.invalid_value(
                "txn_inputs_hash",
                &format!("{h:?}"),
                &format!("Must be equal to the value in Cip509 ({hash:?})"),
                context,
            )
        },
        Err(e) => {
            report.other(
                &format!("Failed to hash transaction inputs: {e:?}"),
                context,
            )
        },
    }
}

/// Checks that the given transaction auxiliary data hash is correct.
pub fn validate_aux(
    auxiliary_data: &[u8], auxiliary_data_hash: Option<&Bytes>, report: &ProblemReport,
) {
    let context = "Cip509 auxiliary data validation";

    let Some(auxiliary_data_hash) = auxiliary_data_hash else {
        report.other("Auxiliary data hash not found in transaction", context);
        return;
    };

    match blake2b_256(auxiliary_data) {
        Ok(h) if h == ***auxiliary_data_hash => {
            // The hash is correct.
        },
        Ok(h) => {
            report.other(
                &format!("Incorrect transaction auxiliary data hash = '{h:?}', expected = '{auxiliary_data_hash:?}'"),
                context,
            )
        },
        Err(e) => {
            report.other(
                &format!("Failed to hash transaction auxiliary data: {e:?}"),
                context,
            )
        },
    }
}

/// Checks that all public keys extracted from x509 and c509 certificates are present in
/// the witness set of the transaction.
pub fn validate_stake_public_key(
    transaction: &MultiEraTx, uris: Option<&Cip0134UriSet>, report: &ProblemReport,
) {
    let context = "Cip509 stake public key validation";

    let witness = match TxWitness::new(&[transaction.clone()]) {
        Ok(w) => w,
        Err(e) => {
            report.other(&format!("Failed to create TxWitness: {e:?}"), context);
            return;
        },
    };

    let pk_addrs = extract_stake_addresses(uris);
    if pk_addrs.is_empty() {
        report.other(
            "Unable to find stake addresses in Cip509 certificates",
            context,
        );
        return;
    }

    if let Err(e) = compare_key_hash(&pk_addrs, &witness, 0) {
        report.other(
            &format!("Failed to compare public keys with witnesses: {e:?}"),
            context,
        );
    }
}

/// Extracts all stake addresses from both X509 and C509 certificates containing in the
/// given `Cip509` and converts their hashes to bytes.
fn extract_stake_addresses(uris: Option<&Cip0134UriSet>) -> Vec<Vec<u8>> {
    let Some(uris) = uris else {
        return Vec::new();
    };

    uris.x_uris()
        .iter()
        .chain(uris.c_uris())
        .flat_map(|(_index, uris)| uris.iter())
        .filter_map(|uri| {
            if let Address::Stake(a) = uri.address() {
                Some(a.payload().as_hash().to_vec())
            } else {
                None
            }
        })
        .collect()
}

/// Checks that the payment key reference is correct and points to a valid key.
pub fn validate_payment_key(
    transaction: &MultiEraTx, conway_transaction: &conway::MintedTx, role_data: &RoleData,
    report: &ProblemReport,
) {
    let context = "Cip509 role0 payment key validation";

    let Some(payment_key) = role_data.payment_key else {
        report.other("Missing payment key in role0", context);
        return;
    };
    if payment_key == 0 {
        report.invalid_value(
            "payment key",
            "0",
            "Payment reference key must not be 0",
            context,
        );
        return;
    }

    // Negative indicates reference to transaction output.
    if payment_key < 0 {
        let index = match decremented_index(payment_key.abs()) {
            Ok(value) => value,
            Err(e) => {
                report.other(
                    &format!("Failed to get index of payment key: {e:?}"),
                    context,
                );
                return;
            },
        };
        let outputs = &conway_transaction.transaction_body.outputs;
        let witness = match TxWitness::new(&[transaction.clone()]) {
            Ok(witnesses) => witnesses,
            Err(e) => {
                report.other(&format!("Failed to create TxWitness: {e:?}"), context);
                return;
            },
        };

        let address = match outputs.get(index) {
            Some(conway::PseudoTransactionOutput::Legacy(o)) => &o.address,
            Some(conway::PseudoTransactionOutput::PostAlonzo(o)) => &o.address,
            None => {
                report.other(
                    &format!("Role payment key reference index ({index}) is not found in transaction outputs"),
                    context,
                );
                return;
            },
        };
        validate_payment_output_key_helper(address, &witness, report, context);
    } else {
        // Positive indicates reference to tx input.
        let inputs = &conway_transaction.transaction_body.inputs;
        let index = match decremented_index(payment_key) {
            Ok(value) => value,
            Err(e) => {
                report.other(
                    &format!("Failed to get index of payment key: {e:?}"),
                    context,
                );
                return;
            },
        };
        // Check whether the index exists in transaction inputs.
        if inputs.get(index).is_none() {
            report.other(
                &format!(
                    "Role payment key reference index ({index}) is not found in transaction inputs"
                ),
                context,
            );
        }
    }
}

/// Helper function for validating payment output key.
fn validate_payment_output_key_helper(
    output_address: &[u8], witness: &TxWitness, report: &ProblemReport, context: &str,
) {
    let Some(key) = extract_key_hash(output_address) else {
        report.other("Failed to extract payment key hash from address", context);
        return;
    };

    // Set transaction index to 0 because the list of transaction is manually constructed
    // for TxWitness -> &[txn.clone()], so we can assume that the witness contains only
    // the witness within this transaction.
    if let Err(e) = compare_key_hash(&[key.clone()], witness, 0) {
        report.other(
            &format!(
                "Unable to find payment output key ({key:?}) in the transaction witness set: {e:?}"
            ),
            context,
        );
    }
}

/// Validate role singing key for role 0.
/// Must reference certificate not the public key
pub fn validate_role_signing_key(role_data: &RoleData, report: &ProblemReport) {
    let Some(ref role_signing_key) = role_data.role_signing_key else {
        return;
    };

    if role_signing_key.local_ref == LocalRefInt::PubKeys {
        report.invalid_value(
            "RoleData::role_signing_key",
            &format!("{role_signing_key:?}"),
            "Role signing key should reference certificate, not public key",
            "Cip509 role0 signing key validation",
        );
    }
}

#[cfg(test)]
mod tests {

    use catalyst_types::problem_report::ProblemReport;
    use minicbor::{Decode, Decoder};
    use pallas::{codec::utils::Nullable, ledger::traverse::MultiEraBlock};

    use super::*;
    use crate::cardano::{cip509::rbac::RoleNumber, transaction::raw_aux_data::RawAuxData};

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
        let conway_block_data = conway_1();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");
        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        let MultiEraTx::Conway(tx) = tx else {
            panic!("Unexpected transaction era");
        };
        let hash = cip509.txn_inputs_hash().unwrap();
        validate_txn_inputs_hash(hash, &tx, &report);
        if report.is_problematic() {
            panic!("validate_txn_inputs_hash failed: {report:?}");
        }
    }

    #[test]
    fn test_validate_aux() {
        let conway_block_data = conway_1();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");
        let MultiEraTx::Conway(tx) = tx else {
            panic!("Unexpected transaction era");
        };
        let auxiliary_data = match &tx.auxiliary_data {
            Nullable::Some(v) => v.raw_cbor(),
            _ => panic!("Missing auxiliary data in transaction"),
        };

        let report = ProblemReport::new("Auxiliary data validation");
        validate_aux(
            auxiliary_data,
            tx.transaction_body.auxiliary_data_hash.as_ref(),
            &report,
        );
        if report.is_problematic() {
            panic!("validate_aux failed: {report:?}");
        }
    }

    #[test]
    fn test_validate_public_key_success() {
        let conway_block_data = conway_1();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        validate_stake_public_key(&tx, cip509.certificate_uris(), &report);
        if report.is_problematic() {
            panic!("validate_stake_public_key failed: {report:?}");
        }
    }

    #[test]
    fn test_validate_public_key_fail() {
        let conway_block_data = conway_2();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .first()
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        validate_stake_public_key(tx, cip509.certificate_uris(), &report);
        assert!(report.is_problematic());
        let report = format!("{report:?}");
        if !report.contains("Failed to compare public keys with witnesses") {
            panic!("Unexpected problem report content: {report}");
        }
    }

    #[test]
    fn test_validate_payment_key_success_negative_ref() {
        let conway_block_data = conway_1();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        let MultiEraTx::Conway(conway_tx) = tx else {
            panic!("Unexpected transaction era");
        };
        let role_data = cip509
            .role_data(RoleNumber::ROLE_0)
            .expect("There must be role0");
        validate_payment_key(tx, conway_tx, role_data, &report);
        if report.is_problematic() {
            panic!("validate_payment_key failed: {report:?}");
        }
    }

    #[test]
    fn test_validate_payment_key_success_positive_ref() {
        let conway_block_data = conway_3();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // First transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .first()
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        let MultiEraTx::Conway(conway_tx) = tx else {
            panic!("Unexpected transaction era");
        };
        let role_data = cip509
            .role_data(RoleNumber::ROLE_0)
            .expect("There must be role0");
        validate_payment_key(tx, conway_tx, role_data, &report);
        if report.is_problematic() {
            panic!("validate_payment_key failed: {report:?}");
        }
    }

    #[test]
    fn test_role_0_signing_key() {
        let conway_block_data = conway_1();
        let multi_era_block =
            MultiEraBlock::decode(&conway_block_data).expect("Failed to decode MultiEraBlock");

        let transactions = multi_era_block.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);

        let mut decoder = Decoder::new(aux_data.as_slice());
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        if report.is_problematic() {
            panic!("Failed to decode Cip509: {report:?}");
        }

        let role_data = cip509
            .role_data(RoleNumber::ROLE_0)
            .expect("There must be role0");
        validate_role_signing_key(role_data, &report);
        if report.is_problematic() {
            panic!("validate_role_signing_key failed: {report:?}");
        }
    }

    #[test]
    fn extract_stake_addresses_from_metadata() {
        let block = conway_1();
        let block = MultiEraBlock::decode(&block).unwrap();

        let transactions = block.txs();
        let tx = transactions
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data = cip_509_aux_data(tx);
        let mut decoder = Decoder::new(&aux_data);
        let mut report = ProblemReport::new("Cip509");
        let cip509 = Cip509::decode(&mut decoder, &mut report).expect("Failed to decode Cip509");
        assert!(!report.is_problematic());

        let addresses = extract_stake_addresses(cip509.certificate_uris());
        // TODO: FIXME:
        println!("{addresses:?}");
        todo!();
    }
}
