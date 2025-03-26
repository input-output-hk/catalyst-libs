//! Functions for extracting public key from X509 and C509 certificates with additional
//! verification.

use anyhow::{anyhow, Context};
use c509_certificate::c509::C509;
use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH};
use oid_registry::{Oid, OID_SIG_ED25519};
use x509_cert::Certificate as X509Certificate;

/// Returns `VerifyingKey` from the given X509 certificate.
///
/// # Errors
///
/// Returns an error if the signature algorithm is not supported and
/// the public key cannot be extracted.
pub fn x509_key(cert: &X509Certificate) -> anyhow::Result<VerifyingKey> {
    let oid: Oid = cert
        .tbs_certificate
        .subject_public_key_info
        .algorithm
        .oid
        .to_string()
        .parse()
        // `Context` cannot be used here because `OidParseError` doesn't implement `std::Error`.
        .map_err(|e| anyhow!("Invalid signature algorithm OID: {e:?}"))?;
    check_signature_algorithm(&oid)?;
    let extended_public_key = cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .as_bytes()
        .context("Invalid subject_public_key value (has unused bits)")?;
    verifying_key(extended_public_key).context("Unable to get verifying key from X509 certificate")
}

/// Returns `VerifyingKey` from the given C509 certificate.
///
/// # Errors
///
/// Returns an error if the signature algorithm is not supported and
/// the public key cannot be extracted.
pub fn c509_key(cert: &C509) -> anyhow::Result<VerifyingKey> {
    let oid = cert
        .tbs_cert()
        .subject_public_key_algorithm()
        .algo_identifier()
        .oid();
    check_signature_algorithm(oid)?;
    verifying_key(cert.tbs_cert().subject_public_key())
        .context("Unable to get verifying key from C509 certificate")
}

/// Checks that the signature algorithm is supported.
fn check_signature_algorithm(oid: &Oid) -> anyhow::Result<()> {
    // Currently the only supported signature algorithm is ED25519.
    if *oid != OID_SIG_ED25519 {
        return Err(anyhow!("Unsupported signature algorithm: {oid}"));
    }
    Ok(())
}

// TODO: The very similar logic exists in the `rbac-registration` crate. It should be
// moved somewhere and reused. See https://github.com/input-output-hk/catalyst-voices/issues/1952
/// Creates `VerifyingKey` from the given extended public key.
fn verifying_key(extended_public_key: &[u8]) -> anyhow::Result<VerifyingKey> {
    /// An extender public key length in bytes.
    const EXTENDED_PUBLIC_KEY_LENGTH: usize = 64;

    if extended_public_key.len() != EXTENDED_PUBLIC_KEY_LENGTH {
        return Err(anyhow!(
            "Unexpected extended public key length in certificate: {}, expected {EXTENDED_PUBLIC_KEY_LENGTH}",
            extended_public_key.len()
        ));
    }
    // This should never fail because of the check above.
    let public_key = extended_public_key
        .get(0..PUBLIC_KEY_LENGTH)
        .context("Unable to get public key part")?;
    let bytes: &[u8; PUBLIC_KEY_LENGTH] = public_key
        .try_into()
        .context("Invalid public key length in X509 certificate")?;
    VerifyingKey::from_bytes(bytes).context("Invalid public key in X509 certificate")
}
