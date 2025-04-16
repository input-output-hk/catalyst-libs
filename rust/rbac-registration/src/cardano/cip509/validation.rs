//! Utilities for validation CIP-0509.
//!
//! See [this document] for all the details.
//!
//! [this document]: https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX

use std::borrow::Cow;

use c509_certificate::c509::C509;
use cardano_blockchain_types::{Network, TxnWitness, VKeyHash};
use catalyst_types::{
    hashes::{Blake2b128Hash, Blake2b256Hash},
    id_uri::IdUri,
    problem_report::ProblemReport,
};
use ed25519_dalek::{Signature, VerifyingKey, PUBLIC_KEY_LENGTH};
use pallas::{
    codec::{
        minicbor::{Encode, Encoder},
        utils::Bytes,
    },
    ledger::{addresses::Address, primitives::conway, traverse::MultiEraTx},
};
use x509_cert::{der::Encode as X509Encode, Certificate as X509};

use super::{
    extract_key::{c509_key, x509_key},
    utils::cip19::compare_key_hash,
};
use crate::cardano::cip509::{
    rbac::Cip509RbacMetadata, types::TxInputHash, C509Cert, Cip0134UriSet, LocalRefInt, RoleData,
    RoleNumber, SimplePublicKeyType, X509DerCert,
};

/// Context-specific primitive type with tag number 6 (`raw_tag` 134) for
/// uniform resource identifier (URI) in the subject alternative name extension.
/// Following the ASN.1
/// <https://www.oss.com/asn1/resources/asn1-made-simple/asn1-quick-reference/asn1-tags.html>
/// the tag is derived from
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

    let calculated_hash = TxInputHash::from(Blake2b128Hash::new(&buffer));
    if &calculated_hash != hash {
        report.invalid_value(
            "txn_inputs_hash",
            &format!("{hash:?}"),
            &format!("Must be equal to the value in Cip509 ({hash:?})"),
            context,
        );
    }
}

/// Checks that the given transaction auxiliary data hash is correct.
pub fn validate_aux(
    raw_aux_data: &[u8], auxiliary_data_hash: Option<&Bytes>, report: &ProblemReport,
) {
    let context = "Cip509 auxiliary data validation";

    let Some(auxiliary_data_hash) = auxiliary_data_hash else {
        report.other("Auxiliary data hash not found in transaction", context);
        return;
    };
    let auxiliary_data_hash = match Blake2b256Hash::try_from(auxiliary_data_hash.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            report.other(
                &format!("Invalid transaction auxiliary data hash: {e:?}"),
                context,
            );
            return;
        },
    };

    let hash = Blake2b256Hash::new(raw_aux_data);
    if hash != auxiliary_data_hash {
        report.other(
            &format!("Incorrect transaction auxiliary data hash = '{hash:?}', expected = '{auxiliary_data_hash:?}'"),
            context,
        );
    }
}

/// Checks that all public keys extracted from x509 and c509 certificates are present in
/// the witness set of the transaction.
pub fn validate_stake_public_key(
    transaction: &conway::MintedTx, uris: Option<&Cip0134UriSet>, report: &ProblemReport,
) {
    let context = "Cip509 stake public key validation";

    let transaction = MultiEraTx::Conway(Box::new(Cow::Borrowed(transaction)));
    let witness = match TxnWitness::new(&[transaction.clone()]) {
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

    if let Err(e) = compare_key_hash(&pk_addrs, &witness, 0.into()) {
        report.other(
            &format!("Failed to compare public keys with witnesses: {e:?}"),
            context,
        );
    }
}

/// Extracts all stake addresses from both X509 and C509 certificates containing in the
/// given `Cip509` and converts their hashes to bytes.
fn extract_stake_addresses(uris: Option<&Cip0134UriSet>) -> Vec<VKeyHash> {
    let Some(uris) = uris else {
        return Vec::new();
    };

    uris.x_uris()
        .iter()
        .chain(uris.c_uris())
        .flat_map(|(_index, uris)| uris.iter())
        .filter_map(|uri| {
            if let Address::Stake(a) = uri.address() {
                a.payload().as_hash().as_slice().try_into().ok()
            } else {
                None
            }
        })
        .collect()
}

/// Validate self-signed certificates.
/// All certificates should be self-signed and support only ED25519 signature.
pub fn validate_self_sign_cert(metadata: &Cip509RbacMetadata, report: &ProblemReport) {
    let context = "Cip509 self-signed certificate validation";

    for (index, cert) in metadata.c509_certs.iter().enumerate() {
        if let C509Cert::C509Certificate(c) = cert {
            validate_c509_self_signed_cert(c, index, report, context);
        }
    }

    for (index, cert) in metadata.x509_certs.iter().enumerate() {
        if let X509DerCert::X509Cert(c) = cert {
            validate_x509_self_signed_cert(c, index, report, context);
        }
    }
}

/// Validate C509 certificate that it is a self-signed.
fn validate_c509_self_signed_cert(c: &C509, index: usize, report: &ProblemReport, context: &str) {
    // Self-sign certificate must be type 2
    if c.tbs_cert().c509_certificate_type() != 2 {
        report.invalid_value(
            &format!("C509 certificate type at index {index}"),
            &c.tbs_cert().c509_certificate_type().to_string(),
            "Certificate must have cert type 2",
            context,
        );
        return;
    }

    let pk = match c509_key(c) {
        Ok(pk) => pk,
        Err(e) => {
            report.other(
                &format!(
                    "Failed to extract subject public key from C509 certificate at index {index}: {e:?}",
                ),
                context,
            );
            return;
        },
    };

    let Some(sig) = c
        .issuer_signature_value()
        .clone()
        .and_then(|b| b.try_into().ok())
        .map(|arr: [u8; 64]| Signature::from_bytes(&arr))
    else {
        report.conversion_error(
            &format!("C509 issuer signature at index {index}"),
            &format!("{:?}", c.issuer_signature_value()),
            "Expected 64-byte Ed25519 signature",
            context,
        );
        return;
    };

    // TODO(bkioshn): signature verification should be improved in c509 crate
    let Ok(tbs_cbor) = c.tbs_cert().to_cbor::<Vec<u8>>() else {
        report.invalid_encoding(
            &format!("C509 TBS certificate at index {index}"),
            "CBOR encoding",
            "Expected CBOR encoded TBS certificate",
            context,
        );
        return;
    };
    if pk.verify_strict(&tbs_cbor, &sig).is_err() {
        report.other(
            &format!("Cannot verify C509 certificate signature at index {index}",),
            context,
        );
    }
}

/// Validate X509 certificate that it is a self-signed.
fn validate_x509_self_signed_cert(c: &X509, index: usize, report: &ProblemReport, context: &str) {
    let pk = match x509_key(c) {
        Ok(pk) => pk,
        Err(e) => {
            report.other(
                &format!(
                    "Failed to extract subject public key from X509 certificate at index {index}: {e:?}",
                ),
                context,
            );
            return;
        },
    };

    let Some(sig) = c
        .signature
        .as_bytes()
        .and_then(|b| b.try_into().ok())
        .map(|arr: [u8; 64]| Signature::from_bytes(&arr))
    else {
        report.conversion_error(
            &format!("X509 signature at index {index}"),
            &format!("{:?}", c.signature.as_bytes()),
            "Expected 64-byte Ed25519 signature",
            context,
        );
        return;
    };

    let tbs_der = match c.tbs_certificate.to_der() {
        Ok(tbs_der) => tbs_der,
        Err(e) => {
            report.invalid_encoding(
                &format!("X509 tbs certificate at index {index}"),
                "DER encoding",
                &format!("Expected DER encoded X509 certificate: {e:?}"),
                context,
            );
            return;
        },
    };

    if pk.verify_strict(&tbs_der, &sig).is_err() {
        report.other(
            &format!("Cannot verify X509 certificate signature at index {index} "),
            context,
        );
    }
}

/// Checks the role data.
#[allow(clippy::similar_names)]
pub fn validate_role_data(
    metadata: &Cip509RbacMetadata, subnet: Network, report: &ProblemReport,
) -> Option<IdUri> {
    let context = "Role data validation";

    // There should be some role data
    if !metadata.role_data.is_empty() {
        if metadata.role_data.contains_key(&RoleNumber::ROLE_0) {
            // For the role 0 there must be exactly once certificate and it must not have `deleted`,
            // `undefined` or `C509CertInMetadatumReference` values.
            if matches!(metadata.x509_certs.first(), Some(X509DerCert::X509Cert(_)))
                && matches!(
                    metadata.c509_certs.first(),
                    Some(C509Cert::C509Certificate(_))
                )
            {
                report.other(
                    "Only one certificate can be defined at index 0 for the role 0",
                    context,
                );
            }
            if !matches!(metadata.x509_certs.first(), Some(X509DerCert::X509Cert(_)))
                && !matches!(
                    metadata.c509_certs.first(),
                    Some(C509Cert::C509Certificate(_))
                )
            {
                report.other("The role 0 certificate must be present", context);
            }
        } else {
            // For other roles there still must be exactly one certificate at 0 index, but it must
            // have the `undefined` value.
            if matches!(metadata.x509_certs.first(), Some(X509DerCert::X509Cert(_)))
                || matches!(
                    metadata.c509_certs.first(),
                    Some(C509Cert::C509Certificate(_))
                )
            {
                report.other("Only role 0 can contain a certificate at 0 index", context);
            }
            if matches!(metadata.x509_certs.first(), Some(X509DerCert::Deleted))
                || matches!(metadata.c509_certs.first(), Some(C509Cert::Deleted))
            {
                report.other("Only role 0 can delete a certificate at 0 index", context);
            }
        }
    }
    // It isn't allowed for any role to use a public key at 0 index.
    if !matches!(
        metadata.pub_keys.first(),
        None | Some(SimplePublicKeyType::Undefined)
    ) {
        report.other(
            "The public key cannot be used at 0 index. Role 0 requires a certificate and other roles must set 0 public key to undefined if needed.",
            context,
        );
    }
    // It isn't allowed for the role 0 to have a certificate in the
    // `C509CertInMetadatumReference` form and other roles must not contain certificate at 0
    // index.
    if matches!(
        metadata.c509_certs.first(),
        Some(C509Cert::C509CertInMetadatumReference(_))
    ) {
        report.other(
            "C509 certificate at 0 index cannot be in metadatum reference",
            context,
        );
    }

    validate_role_numbers(metadata.role_data.keys(), context, report);

    let mut catalyst_id = None;
    for (number, data) in &metadata.role_data {
        if number == &RoleNumber::ROLE_0 {
            catalyst_id = validate_role_0(data, metadata, subnet, context, report);
        } else {
            if let Some(signing_key) = data.signing_key() {
                if signing_key.key_offset == 0 {
                    report.other(
                        &format!(
                            "Invalid signing key: only role 0 can reference a certificate with 0 index ({number:?} {data:?})"
                        ),
                        context,
                    );
                }
            }
            if let Some(encryption_key) = data.encryption_key() {
                if encryption_key.key_offset == 0 {
                    report.other(
                        &format!(
                            "Invalid encryption key: only role 0 can reference a certificate with 0 index ({number:?} {data:?})"
                        ),
                        context,
                    );
                }
            }
        }
    }
    catalyst_id
}

/// Checks that there are no unknown roles.
fn validate_role_numbers<'a>(
    roles: impl Iterator<Item = &'a RoleNumber> + 'a, context: &str, report: &ProblemReport,
) {
    let known_roles = &[RoleNumber::ROLE_0, 3.into()];

    for role in roles {
        if !known_roles.contains(role) {
            report.other(&format!("Unknown role found: {role:?}"), context);
        }
    }
}

/// Checks that the role 0 data is correct.
fn validate_role_0(
    role: &RoleData, metadata: &Cip509RbacMetadata, subnet: Network, context: &str,
    report: &ProblemReport,
) -> Option<IdUri> {
    if let Some(key) = role.encryption_key() {
        report.invalid_value(
            "Role 0 encryption key",
            &format!("{key:?}"),
            "The role 0 shouldn't have the encryption key",
            context,
        );
    }

    let Some(signing_key) = role.signing_key() else {
        report.missing_field("(Role 0) RoleData::signing_key", context);
        return None;
    };

    if signing_key.key_offset != 0 {
        report.other(
            &format!("The role 0 must reference a certificate with 0 index ({role:?})"),
            context,
        );
        return None;
    }

    let mut catalyst_id = None;
    let network = "cardano";

    match signing_key.local_ref {
        LocalRefInt::X509Certs => {
            match metadata.x509_certs.first() {
                Some(X509DerCert::X509Cert(cert)) => {
                    // All good: role 0 references a valid X509 certificate.
                    catalyst_id = x509_cert_key(cert, context, report).map(|k| IdUri::new(network, Some(&subnet.to_string()), k));
                }
                Some(c) => report.other(&format!("Invalid X509 certificate value ({c:?}) for role 0 ({role:?})"), context),
                None => report.other("Role 0 reference X509 certificate at index 0, but there is no such certificate", context),
            }
        },
        LocalRefInt::C509Certs => {
            match metadata.c509_certs.first() {
                Some(C509Cert::C509Certificate(cert)) => {
                    // All good: role 0 references a valid C509 certificate.
                    catalyst_id = c509_cert_key(cert, context, report).map(|k| IdUri::new(network, Some(&subnet.to_string()), k));
                }
                Some(c) => report.other(&format!("Invalid C509 certificate value ({c:?}) for role 0 ({role:?})"), context),
                None => report.other("Role 0 reference C509 certificate at index 0, but there is no such certificate", context),
            }
        },
        LocalRefInt::PubKeys => {
            report.invalid_value(
                "(Role 0) RoleData::signing_key",
                &format!("{signing_key:?}"),
                "Role signing key should reference certificate, not public key",
                context,
            );
        },
    }
    catalyst_id
}

/// Extracts `VerifyingKey` from the given `X509` certificate.
fn x509_cert_key(cert: &X509, context: &str, report: &ProblemReport) -> Option<VerifyingKey> {
    let Some(extended_public_key) = cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .as_bytes()
    else {
        report.invalid_value(
            "subject_public_key",
            "is not octet aligned",
            "Must not have unused bits",
            context,
        );
        return None;
    };
    verifying_key(extended_public_key, context, report)
}

/// Extracts `VerifyingKey` from the given `C509` certificate.
fn c509_cert_key(cert: &C509, context: &str, report: &ProblemReport) -> Option<VerifyingKey> {
    verifying_key(cert.tbs_cert().subject_public_key(), context, report)
}

/// Creates `VerifyingKey` from the given extended public key.
fn verifying_key(
    extended_public_key: &[u8], context: &str, report: &ProblemReport,
) -> Option<VerifyingKey> {
    /// An extender public key length in bytes.
    const EXTENDED_PUBLIC_KEY_LENGTH: usize = 64;

    if extended_public_key.len() != EXTENDED_PUBLIC_KEY_LENGTH {
        report.other(
            &format!("Unexpected extended public key length in certificate: {}, expected {EXTENDED_PUBLIC_KEY_LENGTH}",
                extended_public_key.len()),
            context,
        );
        return None;
    }

    // This should never fail because of the check above.
    let Some(public_key) = extended_public_key.get(0..PUBLIC_KEY_LENGTH) else {
        report.other("Unable to get public key part", context);
        return None;
    };

    let bytes: &[u8; PUBLIC_KEY_LENGTH] = match public_key.try_into() {
        Ok(v) => v,
        Err(e) => {
            report.other(
                &format!("Invalid public key length in X509 certificate: {e:?}"),
                context,
            );
            return None;
        },
    };
    match VerifyingKey::from_bytes(bytes) {
        Ok(k) => Some(k),
        Err(e) => {
            report.other(
                &format!("Invalid public key in C509 certificate: {e:?}"),
                context,
            );
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cardano::cip509::Cip509, utils::test};

    #[test]
    fn block_1() {
        let data = test::block_1();

        let mut registrations = Cip509::from_block(&data.block, &[]);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        data.assert_valid(&registration);
    }

    #[test]
    fn block_2() {
        let data = test::block_2();

        let mut registrations = Cip509::from_block(&data.block, &[]);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(registration.report().is_problematic());

        let origin = registration.origin();
        assert_eq!(origin.txn_index(), data.txn_index);
        assert_eq!(origin.point().slot_or_default(), data.slot);

        // The consume function must return the problem report contained within the registration.
        let report = registration.consume().unwrap_err();
        assert!(report.is_problematic());
        let report = format!("{report:?}");
        assert!(report.contains("Public key hash not found in transaction witness set"));
    }

    #[test]
    fn block_3() {
        let data = test::block_3();

        let mut registrations = Cip509::from_block(&data.block, &[]);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(registration.report().is_problematic());

        assert_eq!(registration.previous_transaction(), data.prv_hash);

        let origin = registration.origin();
        assert_eq!(origin.txn_index(), data.txn_index);
        assert_eq!(origin.point().slot_or_default(), data.slot);

        let report = registration.consume().unwrap_err();
        assert!(report.is_problematic());
        let report = format!("{report:?}");
        assert!(report
            .contains("Role payment key reference index (1) is not found in transaction outputs"));
    }

    #[test]
    fn block_4() {
        let data = test::block_4();

        let mut registrations = Cip509::from_block(&data.block, &[]);
        assert_eq!(1, registrations.len());

        let registration = registrations.pop().unwrap();
        assert!(registration.report().is_problematic());

        let origin = registration.origin();
        assert_eq!(origin.txn_index(), data.txn_index);
        assert_eq!(origin.point().slot_or_default(), data.slot);

        // The consume function must return the problem report contained within the registration.
        let report = registration.consume().unwrap_err();
        assert!(report.is_problematic());
        let report = format!("{report:?}");
        assert!(report.contains("Unknown role found: RoleNumber(4)"));
    }

    #[test]
    fn extract_stake_addresses_from_metadata() {
        let data = test::block_1();
        let cip509 = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
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
        let hash = address.payload().as_hash().as_ref().try_into().unwrap();

        let addresses = extract_stake_addresses(cip509.certificate_uris());
        assert_eq!(1, addresses.len());
        assert_eq!(addresses.first().unwrap(), &hash);
    }
}
