//! Providers traits, which are used during different validation procedures.

use std::time::Duration;

use catalyst_types::{catalyst_id::CatalystId, uuid::UuidV7};
use ed25519_dalek::VerifyingKey;

use crate::{CatalystSignedDocument, DocumentRef};

/// `CatalystId` Provider trait
#[async_trait::async_trait]
pub trait CatalystIdProvider: Send + Sync {
    /// Try to get `VerifyingKey` by the provided `CatalystId` and corresponding `RoleId`
    /// and `KeyRotation` Return `None` if the provided `CatalystId` with the
    /// corresponding `RoleId` and `KeyRotation` has not been registered.
    async fn try_get_registered_key(
        &self,
        kid: &CatalystId,
    ) -> anyhow::Result<Option<VerifyingKey>>;
}

/// `CatalystSignedDocument` Provider trait
#[async_trait::async_trait]
pub trait CatalystSignedDocumentProvider: Send + Sync {
    /// Try to get `CatalystSignedDocument` from document reference
    async fn try_get_doc(
        &self,
        doc_ref: &DocumentRef,
    ) -> anyhow::Result<Option<CatalystSignedDocument>>;

    /// Try to get the last known version of the `CatalystSignedDocument`, same
    /// `id` and the highest known `ver`.
    async fn try_get_last_doc(
        &self,
        id: UuidV7,
    ) -> anyhow::Result<Option<CatalystSignedDocument>>;

    /// Try to get the first known version of the `CatalystSignedDocument`, `id` and `ver`
    /// are equal.
    async fn try_get_first_doc(
        &self,
        id: UuidV7,
    ) -> anyhow::Result<Option<CatalystSignedDocument>>;

    /// Returns a future threshold value, which is used in the validation of the `ver`
    /// field that it is not too far in the future.
    /// If `None` is returned, skips "too far in the future" validation.
    fn future_threshold(&self) -> Option<Duration>;

    /// Returns a past threshold value, which is used in the validation of the `ver`
    /// field that it is not too far behind.
    /// If `None` is returned, skips "too far behind" validation.
    fn past_threshold(&self) -> Option<Duration>;
}

/// Super trait of `CatalystSignedDocumentProvider` and `CatalystIdProvider`
pub trait CatalystSignedDocumentAndCatalystIdProvider:
    CatalystSignedDocumentProvider + CatalystIdProvider
{
}

impl<T: CatalystSignedDocumentProvider + CatalystIdProvider>
    CatalystSignedDocumentAndCatalystIdProvider for T
{
}

pub mod tests {
    //! Simple providers implementation just for the testing purposes

    use std::{collections::HashMap, time::Duration};

    use ed25519_dalek::SigningKey;

    use super::{
        CatalystId, CatalystIdProvider, CatalystSignedDocument, CatalystSignedDocumentProvider,
        VerifyingKey,
    };
    use crate::DocumentRef;

    /// Simple testing implementation of `CatalystSignedDocumentProvider`,
    #[derive(Default, Debug)]
    pub struct TestCatalystProvider {
        /// For `CatalystSignedDocumentProvider`.
        signed_doc: HashMap<DocumentRef, CatalystSignedDocument>,
        /// For `VerifyingKeyProvider`.
        secret_key: HashMap<CatalystId, SigningKey>,
    }

    impl TestCatalystProvider {
        /// Inserts document into the `TestCatalystSignedDocumentProvider`.
        ///
        /// # Errors
        /// Returns error if document reference is not provided and its fail to create one
        /// from the given doc.
        pub fn add_document(
            &mut self,
            doc: &CatalystSignedDocument,
        ) -> anyhow::Result<()> {
            let dr = doc.doc_ref()?;
            self.signed_doc.insert(dr, doc.clone());
            Ok(())
        }

        /// Inserts document into the `TestCatalystSignedDocumentProvider` using provided
        /// `DocumentRef` as key.
        pub fn add_document_with_ref(
            &mut self,
            doc_ref: DocumentRef,
            doc: &CatalystSignedDocument,
        ) {
            self.signed_doc.insert(doc_ref, doc.clone());
        }

        /// Inserts signing key into the `TestVerifyingKeyProvider`
        pub fn add_sk(
            &mut self,
            kid: CatalystId,
            sk: SigningKey,
        ) {
            self.secret_key.insert(kid, sk);
        }

        /// Returns a reference to the corresponding `SigningKey`.
        #[must_use]
        pub fn get_sk(
            &self,
            kid: &CatalystId,
        ) -> Option<&SigningKey> {
            self.secret_key.get(kid)
        }
    }

    #[async_trait::async_trait]
    impl CatalystSignedDocumentProvider for TestCatalystProvider {
        async fn try_get_doc(
            &self,
            doc_ref: &DocumentRef,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self.signed_doc.get(doc_ref).cloned())
        }

        async fn try_get_last_doc(
            &self,
            id: catalyst_types::uuid::UuidV7,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self
                .signed_doc
                .iter()
                .filter(|(doc_ref, _)| doc_ref.id() == &id)
                .max_by_key(|(doc_ref, _)| doc_ref.ver().uuid())
                .map(|(_, doc)| doc.clone()))
        }

        async fn try_get_first_doc(
            &self,
            id: catalyst_types::uuid::UuidV7,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self
                .signed_doc
                .iter()
                .filter(|(doc_ref, _)| doc_ref.id() == &id)
                .min_by_key(|(doc_ref, _)| doc_ref.ver().uuid())
                .map(|(_, doc)| doc.clone()))
        }

        fn future_threshold(&self) -> Option<std::time::Duration> {
            Some(Duration::from_secs(5))
        }

        fn past_threshold(&self) -> Option<Duration> {
            Some(Duration::from_secs(5))
        }
    }

    #[async_trait::async_trait]
    impl CatalystIdProvider for TestCatalystProvider {
        async fn try_get_registered_key(
            &self,
            kid: &CatalystId,
        ) -> anyhow::Result<Option<VerifyingKey>> {
            Ok(self.secret_key.get(kid).map(SigningKey::verifying_key))
        }
    }
}
