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

use std::borrow::Cow;

use catalyst_types::problem_report::ProblemReport;
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::{addresses::Address, primitives::conway, traverse::MultiEraTx},
};

use super::utils::cip19::compare_key_hash;
use crate::{
    cardano::{
        cip509::{types::TxInputHash, Cip0134UriSet, LocalRefInt, RoleData},
        transaction::witness::TxWitness,
    },
    utils::hashing::{blake2b_128, blake2b_256},
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
            );
        },
        Err(e) => {
            report.other(
                &format!("Failed to hash transaction inputs: {e:?}"),
                context,
            );
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
            );
        },
        Err(e) => {
            report.other(
                &format!("Failed to hash transaction auxiliary data: {e:?}"),
                context,
            );
        },
    }
}

/// Checks that all public keys extracted from x509 and c509 certificates are present in
/// the witness set of the transaction.
pub fn validate_stake_public_key(
    transaction: &conway::MintedTx, uris: Option<&Cip0134UriSet>, report: &ProblemReport,
) {
    let context = "Cip509 stake public key validation";

    let transaction = MultiEraTx::Conway(Box::new(Cow::Borrowed(transaction)));
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

/// Validate role singing key for role 0.
/// Must reference certificate not the public key
pub fn validate_role_signing_key(role_data: &RoleData, report: &ProblemReport) {
    let Some(role_signing_key) = role_data.signing_key() else {
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
    use std::str::FromStr;

    use cardano_blockchain_types::hashes::Blake2b256Hash;
    use pallas::ledger::traverse::MultiEraBlock;
    use uuid::Uuid;

    use super::*;
    use crate::cardano::cip509::Cip509;

    // `conway_1.block` contains one transaction (index = 3) with the `Cip509` data.
    #[test]
    fn block_1() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_1.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();

        let mut registrations = Cip509::from_block(&block);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(
            !registration.report().is_problematic(),
            "{:?}",
            registration.report()
        );
        assert!(registration.previous_transaction().is_none());
        assert_eq!(registration.origin(), (77_429_134.into(), 3.into()));

        let (purpose, metadata) = registration.consume().unwrap();
        assert_eq!(
            purpose,
            Uuid::parse_str("ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c").unwrap()
        );
        assert_eq!(1, metadata.role_data.len());
    }

    // `conway_2.block` contains  one transaction (index = 0) with the `Cip509` data. Also
    // this registration contains an invalid public key that isn't present in the transaction
    // witness set.
    #[test]
    fn block_2() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_2.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();

        let mut registrations = Cip509::from_block(&block);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(registration.report().is_problematic());
        assert_eq!(registration.origin(), (77_171_632.into(), 0.into()));

        // The consume function must return the problem report contained within the registration.
        let report = registration.consume().unwrap_err();
        assert!(report.is_problematic());
        let report = format!("{report:?}");
        assert!(report.contains("Public key hash not found in transaction witness set"));
    }

    // `conway_3.block` contains one transaction (index = 0) with the `Cip509` data.
    #[test]
    fn block_3() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_3.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();

        let mut registrations = Cip509::from_block(&block);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(
            !registration.report().is_problematic(),
            "{:?}",
            registration.report()
        );
        assert_eq!(
            registration.previous_transaction(),
            Some(
                Blake2b256Hash::from_str(
                    "4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2"
                )
                .unwrap()
            )
        );
        assert_eq!(registration.origin(), (77_170_639.into(), 0.into()));

        let (purpose, metadata) = registration.consume().unwrap();
        assert_eq!(
            purpose,
            Uuid::parse_str("ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c").unwrap()
        );
        assert_eq!(1, metadata.role_data.len());
    }

    // `conway_3.block` contains one transaction (index = 1) with the `Cip509` data.
    #[test]
    fn block_4() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_4.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();

        let mut registrations = Cip509::from_block(&block);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(
            !registration.report().is_problematic(),
            "{:?}",
            registration.report()
        );
        assert_eq!(
            registration.previous_transaction(),
            Some(
                Blake2b256Hash::from_str(
                    "6695b9cac9230af5c8ee50747b1ca3c78a854d181c7e5c6c371de01b80274d31"
                )
                .unwrap()
            )
        );
        assert_eq!(registration.origin(), (77_436_369.into(), 1.into()));

        let (purpose, metadata) = registration.consume().unwrap();
        assert_eq!(
            purpose,
            Uuid::parse_str("ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c").unwrap()
        );
        assert_eq!(1, metadata.role_data.len());
    }

    #[test]
    fn extract_stake_addresses_from_metadata() {
        let block = hex::decode(include_str!("../../test_data/cardano/conway_1.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();
        let cip509 = Cip509::new(&block, 3.into()).unwrap().unwrap();
        assert!(
            !cip509.report().is_problematic(),
            "Failed to decode Cip509: {:?}",
            cip509.report()
        );

        let uris = cip509.certificate_uris().unwrap();
        assert!(uris.c_uris().is_empty());
        assert_eq!(1, uris.x_uris().len());
        let Address::Stake(address) = uris.x_uris().get(&0).unwrap().first().unwrap().address()
        else {
            panic!("Unexpected address type");
        };
        let hash = address.payload().as_hash().to_vec();

        let addresses = extract_stake_addresses(cip509.certificate_uris());
        assert_eq!(1, addresses.len());
        assert_eq!(addresses.first().unwrap(), &hash);
    }
}
