//! Functions for extracting public key from X509 and C509 certificates with additional
//! verification.

use anyhow::anyhow;
use c509_certificate::c509::C509;
use catalyst_types::problem_report::ProblemReport;
use ed25519_dalek::{SignatureError, VerifyingKey, PUBLIC_KEY_LENGTH};
use oid_registry::{Oid, OID_SIG_ED25519};
use thiserror::Error;
use x509_cert::Certificate as X509Certificate;

/// An extended public key length in bytes.
const EXTENDED_PUBLIC_KEY_LENGTH: usize = 64;

/// Common error type.
#[derive(Error, Debug)]
pub enum Error {
    /// Unsupported signature algorithms.
    #[error("Unsupported signature algorithm: {oid}")]
    UnsupportedSignatureAlgo {
        /// An OID of unsupported signature algorithm.
        oid: String,
    },
    /// Incorrect extended public key byte length.
    #[error("Unexpected extended public key length in certificate: {0}, expected {EXTENDED_PUBLIC_KEY_LENGTH}")]
    InvalidExtendedPublicKeyLength(usize),
    /// Public key has unused bits in a bit string.
    #[error("Invalid subject_public_key value (has unused bits)")]
    PublicKeyHasUnusedBits,
    /// Public key cannot be converted into a [`VerifyingKey`].
    #[error("Cannot create verifying key from subject_public_key: {0}")]
    PublicKeyIsNotVerifyingKey(#[from] SignatureError),
    /// Unexpected error.
    #[error("Unexpected error: {0}")]
    Unexpected(#[source] anyhow::Error),
}

impl Error {
    /// Shortcut function to report `Self` into a [`ProblemReport`].
    pub fn report_problem(&self, context: &str, report: &ProblemReport) {
        match self {
            Error::UnsupportedSignatureAlgo { oid } => {
                report.invalid_value(
                    "subject_public_key_algorithm",
                    oid,
                    "Currently the only supported signature algorithm is ED25519",
                    context,
                );
            },
            Error::InvalidExtendedPublicKeyLength(found) => {
                report.invalid_value(
                    "subject_public_key",
                    &format!("{found} bytes"),
                    &format!("Must be {EXTENDED_PUBLIC_KEY_LENGTH} bytes"),
                    context,
                );
            },
            Error::PublicKeyHasUnusedBits => {
                report.invalid_value(
                    "subject_public_key",
                    "is not octet aligned",
                    "Must not have unused bits",
                    context,
                );
            },
            Error::PublicKeyIsNotVerifyingKey(error) => {
                report.invalid_value(
                    "subject_public_key",
                    &error.to_string(),
                    "Must be a verifying key",
                    context,
                );
            },
            Error::Unexpected(error) => report.other(&error.to_string(), context),
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
    let oid_string = cert
        .tbs_certificate
        .subject_public_key_info
        .algorithm
        .oid
        .to_string();
    let oid: Oid = oid_string
        .parse()
        .map_err(|_| Error::UnsupportedSignatureAlgo { oid: oid_string })?;
    check_signature_algorithm(&oid)?;
    let extended_public_key = cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .as_bytes()
        .ok_or(Error::PublicKeyHasUnusedBits)?;
    verifying_key(extended_public_key)
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
    verifying_key(cert.tbs_cert().subject_public_key())
}

/// Checks that the signature algorithm is supported.
fn check_signature_algorithm(oid: &Oid) -> Result<(), Error> {
    // Currently the only supported signature algorithm is ED25519.
    if *oid != OID_SIG_ED25519 {
        return Err(Error::UnsupportedSignatureAlgo {
            oid: oid.to_id_string(),
        });
    }
    Ok(())
}

/// Creates `VerifyingKey` from the given extended public key.
fn verifying_key(extended_public_key: &[u8]) -> Result<VerifyingKey, Error> {
    if extended_public_key.len() != EXTENDED_PUBLIC_KEY_LENGTH {
        return Err(Error::InvalidExtendedPublicKeyLength(
            extended_public_key.len(),
        ));
    }
    // This should never fail because of the check above.
    let bytes = extended_public_key
        .first_chunk::<PUBLIC_KEY_LENGTH>()
        .ok_or_else(|| {
            Error::Unexpected(anyhow!(
                "Public key part length {PUBLIC_KEY_LENGTH} must be less \
                than the extended length {EXTENDED_PUBLIC_KEY_LENGTH}"
            ))
        })?;

    VerifyingKey::from_bytes(bytes).map_err(Error::PublicKeyIsNotVerifyingKey)
}
