//! Providers traits, which are used during different validation procedures.

use std::future::Future;

use catalyst_types::id_uri::IdUri;
use ed25519_dalek::VerifyingKey;

use crate::{CatalystSignedDocument, DocumentRef};

/// `VerifyingKey` Provider trait
pub trait VerifyingKeyProvider {
    /// Try to get `VerifyingKey`
    fn try_get_key(
        &self, kid: &IdUri,
    ) -> impl Future<Output = anyhow::Result<Option<VerifyingKey>>>;
}

/// `CatalystSignedDocument` Provider trait
pub trait CatalystSignedDocumentProvider: Send + Sync {
    /// Try to get `CatalystSignedDocument`
    fn try_get_doc(
        &self, doc_ref: &DocumentRef,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystSignedDocument>>> + Send;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) struct TestCatalystSignedDocumentProvider<F>(pub(crate) F)
    where F: Fn(&DocumentRef) -> anyhow::Result<Option<CatalystSignedDocument>> + Send + Sync;

    impl<F> CatalystSignedDocumentProvider for TestCatalystSignedDocumentProvider<F>
    where F: Fn(&DocumentRef) -> anyhow::Result<Option<CatalystSignedDocument>> + Send + Sync
    {
        async fn try_get_doc(
            &self, doc_ref: &DocumentRef,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            self.0(doc_ref)
        }
    }
}
