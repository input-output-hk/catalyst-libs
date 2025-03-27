//! C509 certificate, X509 certificate, or public key.

use std::sync::Arc;

use c509_certificate::c509::C509;
use ed25519_dalek::VerifyingKey;
use x509_cert::Certificate as X509;

use crate::cardano::cip509::extract_key;

/// Actual data of key local ref. Containing X509, C509 and public key.
#[derive(Debug, Clone)]
pub enum CertOrPk {
    /// X509 certificate, None if deleted.
    X509(Option<Arc<X509>>),
    /// C509 certificate, None if deleted.
    C509(Option<Arc<C509>>),
    /// Public key, None if deleted.
    PublicKey(Option<VerifyingKey>),
}

impl CertOrPk {
    /// Extract public key from the given certificate or public key.
    pub(crate) fn extract_pk(&self) -> Option<VerifyingKey> {
        match self {
            CertOrPk::X509(Some(x509)) => extract_key::x509_key(x509).ok(),
            CertOrPk::C509(Some(c509)) => extract_key::c509_key(c509).ok(),
            CertOrPk::PublicKey(pk) => *pk,
            _ => None,
        }
    }
}
