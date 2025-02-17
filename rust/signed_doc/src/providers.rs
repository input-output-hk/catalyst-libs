//! Providers traits, which are used during different validation procedures.

use std::future::Future;

use catalyst_types::id_uri::IdUri;
use ed25519_dalek::VerifyingKey;

/// `VerifyingKey` Provider trait
pub trait VerifyingKeyProvider {
    /// Try to get `VerifyingKey`
    fn try_get_vk(&self, kid: &IdUri)
        -> impl Future<Output = anyhow::Result<Option<VerifyingKey>>>;
}
