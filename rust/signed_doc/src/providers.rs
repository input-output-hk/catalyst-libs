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
    use std::collections::HashMap;

    use catalyst_types::uuid::Uuid;

    use super::*;

    /// Index documents only by `id` field
    #[derive(Default)]
    pub(crate) struct TestCatalystSignedDocumentProvider(HashMap<Uuid, CatalystSignedDocument>);

    impl TestCatalystSignedDocumentProvider {
        pub(crate) fn add_document(&mut self, doc: CatalystSignedDocument) -> anyhow::Result<()> {
            self.0.insert(doc.doc_id()?.uuid(), doc);
            Ok(())
        }
    }

    impl CatalystSignedDocumentProvider for TestCatalystSignedDocumentProvider {
        async fn try_get_doc(
            &self, doc_ref: &DocumentRef,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self.0.get(&doc_ref.id.uuid()).cloned())
        }
    }
}
