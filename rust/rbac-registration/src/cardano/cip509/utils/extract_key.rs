//! Functions for extracting public key from X509 and C509 certificates with additional
//! verification.

use std::borrow::Cow;

use c509_certificate::c509::C509;
use catalyst_types::problem_report::ProblemReport;
use ed25519_dalek::{SignatureError, VerifyingKey, PUBLIC_KEY_LENGTH};
use oid_registry::{Oid, OID_SIG_ED25519};
use thiserror::Error;
use x509_cert::{spki, Certificate as X509Certificate};

/// Common error type.
#[derive(Debug, Error)]
pub enum Error {
    /// Unsupported signature algorithms.
    #[error("Unsupported signature algorithm (oid: {oid})")]
    UnsupportedSignatureAlgo {
        /// An OID of unsupported signature algorithm.
        oid: Oid<'static>,
    },
    /// Public key has invalid length.
    #[error(
        "Invalid public key length (found {bytes} bytes, but expected {PUBLIC_KEY_LENGTH} bytes)"
    )]
    InvalidPublicKeyLength {
        /// Number of bytes found.
        bytes: usize,
    },
    /// Public key is stored in a bit string, where number of unused bits is *not* equal
    /// to zero.
    #[error("Invalid public key is not octet aligned (found {bits} bits)")]
    PublicKeyIsNotOctetAligned {
        /// Number of bits found.
        bits: usize,
    },
    /// Public key doesn't pass [`ed25519_dalek`] constraint check.
    #[error("Invalid public key ({source})")]
    PublicKeyIsNotEd25519 {
        /// Underlying [`ed25519_dalek`] error.
        #[from]
        source: SignatureError,
    },
}

impl Error {
    /// Shortcut function to report `Self` into a [`ProblemReport`].
    pub fn report_problem(&self, context: &str, report: &ProblemReport) {
        match self {
            Error::UnsupportedSignatureAlgo { oid } => {
                report.invalid_value(
                    "subject_public_key_algorithm",
                    &oid.to_id_string(),
                    "Currently the only supported signature algorithm is ED25519",
                    context,
                );
            },
            Error::InvalidPublicKeyLength { bytes } => {
                report.invalid_value(
                    "subject_public_key",
                    &format!("{bytes} bytes"),
                    &format!("Must be {PUBLIC_KEY_LENGTH} bytes long"),
                    context,
                );
            },
            Error::PublicKeyIsNotOctetAligned { bits } => {
                report.invalid_value(
                    "subject_public_key",
                    &format!("{bits} bits"),
                    "Bit string must be octet aligned having no unused bits",
                    context,
                );
            },
            Error::PublicKeyIsNotEd25519 { source } => {
                report.invalid_value(
                    "subject_public_key",
                    &source.to_string(),
                    "Must be an Ed25519 public key",
                    context,
                );
            },
        }
    }
}

/// Returns `VerifyingKey` from the given X509 certificate.
///
/// # Errors
///
/// Returns an error if the signature algorithm is not supported and
/// the public key cannot be extracted.
///
/// Returns an error if public key has unexpected value.
pub fn x509_key(cert: &X509Certificate) -> Result<VerifyingKey, Error> {
    let oid = &cert.tbs_certificate.subject_public_key_info.algorithm.oid;
    check_signature_algorithm(&spki_oid_as_asn1_rs_oid(oid))?;
    let public_key = &cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key;
    let public_key_bytes = public_key
        .as_bytes()
        .ok_or(Error::PublicKeyIsNotOctetAligned {
            bits: public_key.bit_len(),
        })?;
    verifying_key(public_key_bytes)
}

/// Returns `VerifyingKey` from the given C509 certificate.
///
/// # Errors
///
/// Returns an error if the signature algorithm is not supported and
/// the public key cannot be extracted.
///
/// Returns an error if public key has unexpected value.
pub fn c509_key(cert: &C509) -> Result<VerifyingKey, Error> {
    let oid = cert
        .tbs_cert()
        .subject_public_key_algorithm()
        .algo_identifier()
        .oid();
    check_signature_algorithm(oid)?;
    let public_key = cert.tbs_cert().subject_public_key();
    verifying_key(public_key)
}

/// Checks that the signature algorithm with the given [`spki::ObjectIdentifier`] is
/// supported.
fn check_signature_algorithm(oid: &Oid) -> Result<(), Error> {
    // Currently the only supported signature algorithm is ED25519.
    if *oid == OID_SIG_ED25519 {
        Ok(())
    } else {
        Err(Error::UnsupportedSignatureAlgo {
            oid: oid.to_owned(),
        })
    }
}

/// Converts [`spki::ObjectIdentifier`] ref to an [`Oid`].
fn spki_oid_as_asn1_rs_oid(oid: &'_ spki::ObjectIdentifier) -> Oid<'_> {
    // Note that this conversion always succeeds as both crates omit header.
    Oid::new(Cow::Borrowed(oid.as_bytes()))
}

/// Creates [`VerifyingKey`] from the first 32 bytes in a slice.
/// Since only prefix bytes are used, both extended and common public keys are supported
/// here.
fn verifying_key(public_key: &[u8]) -> Result<VerifyingKey, Error> {
    public_key
        // TODO: replace with checked `[u8; 32]` conversion once we only support common ed25119.
        .first_chunk()
        // Public key is too short.
        .ok_or(Error::InvalidPublicKeyLength {
            bytes: public_key.len(),
        })
        .and_then(|bytes| VerifyingKey::from_bytes(bytes).map_err(Error::from))
}

#[cfg(test)]
mod tests {
    use oid_registry::{asn1_rs, OID_SIG_ED25519};
    use x509_cert::spki;

    #[test]
    fn spki_oid_as_asn1_rs_oid() {
        let spki_oid = spki::ObjectIdentifier::new_unwrap("1.3.101.112");
        let asn1_rs_oid = asn1_rs::oid!(1.3.101 .112);
        assert_eq!(spki_oid.to_string(), asn1_rs_oid.to_id_string());

        let converted_spki_oid = super::spki_oid_as_asn1_rs_oid(&spki_oid);
        assert_eq!(converted_spki_oid.to_string(), asn1_rs_oid.to_id_string());
        assert_eq!(converted_spki_oid, OID_SIG_ED25519);
    }
}
