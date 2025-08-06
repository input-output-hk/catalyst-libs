//! Providers traits, which are used during different validation procedures.

use std::{future::Future, time::Duration};

use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::VerifyingKey;

use crate::{CatalystSignedDocument, DocumentRef};

/// `VerifyingKey` Provider trait
pub trait VerifyingKeyProvider {
    /// Try to get `VerifyingKey`
    fn try_get_key(
        &self,
        kid: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<Option<VerifyingKey>>>;
}

/// `CatalystSignedDocument` Provider trait
pub trait CatalystSignedDocumentProvider: Send + Sync {
    /// Try to get `CatalystSignedDocument`from document reference
    fn try_get_doc(
        &self,
        doc_ref: &DocumentRef,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystSignedDocument>>> + Send;

    /// Returns a future threshold value, which is used in the validation of the `ver`
    /// field that it is not too far in the future.
    /// If `None` is returned, skips "too far in the future" validation.
    fn future_threshold(&self) -> Option<Duration>;

    /// Returns a past threshold value, which is used in the validation of the `ver`
    /// field that it is not too far behind.
    /// If `None` is returned, skips "too far behind" validation.
    fn past_threshold(&self) -> Option<Duration>;
}

pub mod tests {
    //! Simple providers implementation just for the testing purposes

    use std::{collections::HashMap, time::Duration};

    use super::{
        CatalystId, CatalystSignedDocument, CatalystSignedDocumentProvider, VerifyingKey,
        VerifyingKeyProvider,
    };
    use crate::{DocLocator, DocumentRef};

    ///  Simple testing implementation of `CatalystSignedDocumentProvider`
    #[derive(Default, Debug)]

    pub struct TestCatalystSignedDocumentProvider(HashMap<DocumentRef, CatalystSignedDocument>);

    impl TestCatalystSignedDocumentProvider {
        /// Inserts document into the `TestCatalystSignedDocumentProvider` where
        /// if document reference is provided use that value.
        /// if not use the id and version of the provided doc.
        ///
        /// # Errors
        /// Returns error if document reference is not provided and its fail to create one
        /// from the given doc.
        pub fn add_document(
            &mut self,
            doc_ref: Option<DocumentRef>,
            doc: &CatalystSignedDocument,
        ) -> anyhow::Result<()> {
            if let Some(dr) = doc_ref {
                self.0.insert(dr, doc.clone());
            } else {
                let dr = DocumentRef::new(doc.doc_id()?, doc.doc_ver()?, DocLocator::default());
                self.0.insert(dr, doc.clone());
            }
            Ok(())
        }
    }

    impl CatalystSignedDocumentProvider for TestCatalystSignedDocumentProvider {
        async fn try_get_doc(
            &self,
            doc_ref: &DocumentRef,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self.0.get(doc_ref).cloned())
        }

        fn future_threshold(&self) -> Option<std::time::Duration> {
            Some(Duration::from_secs(5))
        }

        fn past_threshold(&self) -> Option<Duration> {
            Some(Duration::from_secs(5))
        }
    }

    /// Simple testing implementation of `VerifyingKeyProvider`
    #[derive(Default)]
    pub struct TestVerifyingKeyProvider(HashMap<CatalystId, VerifyingKey>);

    impl TestVerifyingKeyProvider {
        /// Inserts public key into the `TestVerifyingKeyProvider`
        pub fn add_pk(
            &mut self,
            kid: CatalystId,
            pk: VerifyingKey,
        ) {
            self.0.insert(kid, pk);
        }
    }

    impl VerifyingKeyProvider for TestVerifyingKeyProvider {
        async fn try_get_key(
            &self,
            kid: &CatalystId,
        ) -> anyhow::Result<Option<VerifyingKey>> {
            Ok(self.0.get(kid).copied())
        }
    }
}
