//! Providers traits, which are used during different validation procedures.

use std::{future::Future, time::Duration};

use catalyst_types::{catalyst_id::CatalystId, uuid::UuidV7};
use ed25519_dalek::VerifyingKey;

use crate::{CatalystSignedDocument, DocumentRef};

/// `CatalystId` Provider trait
pub trait CatalystIdProvider: Send + Sync {
    /// Try to get `VerifyingKey` by the provided `CatalystId` and corresponding `RoleId`
    /// and `KeyRotation` Return `None` if the provided `CatalystId` with the
    /// corresponding `RoleId` and `KeyRotation` has not been registered.
    fn try_get_registered_key(
        &self,
        kid: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<Option<VerifyingKey>>> + Send;
}

/// `CatalystSignedDocument` Provider trait
pub trait CatalystSignedDocumentProvider: Send + Sync {
    /// Try to get `CatalystSignedDocument` from document reference
    fn try_get_doc(
        &self,
        doc_ref: &DocumentRef,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystSignedDocument>>> + Send;

    /// Try to get all versions of the `CatalystSignedDocument` from the given `id`.
    fn try_get_all(
        &self,
        id: UuidV7,
    ) -> impl Future<Output = anyhow::Result<Vec<CatalystSignedDocument>>> + Send;

    /// Try to get the last known version of the `CatalystSignedDocument`, same
    /// `id` and the highest known `ver`.
    fn try_get_last_doc(
        &self,
        id: UuidV7,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystSignedDocument>>> + Send;

    /// Try to get the first known version of the `CatalystSignedDocument`, `id` and `ver`
    /// are equal.
    fn try_get_first_doc(
        &self,
        id: UuidV7,
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
        CatalystId, CatalystIdProvider, CatalystSignedDocument, CatalystSignedDocumentProvider,
        VerifyingKey,
    };
    use crate::{DocLocator, DocumentRef};

    /// Simple testing implementation of `CatalystSignedDocumentProvider`,
    #[derive(Default, Debug)]
    pub struct TestCatalystProvider {
        /// For `CatalystSignedDocumentProvider`.
        signed_doc: HashMap<DocumentRef, CatalystSignedDocument>,
        /// For `VerifyingKeyProvider`.
        verifying_key: HashMap<CatalystId, VerifyingKey>,
    }

    impl TestCatalystProvider {
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
                self.signed_doc.insert(dr, doc.clone());
            } else {
                let dr = DocumentRef::new(doc.doc_id()?, doc.doc_ver()?, DocLocator::default());
                self.signed_doc.insert(dr, doc.clone());
            }
            Ok(())
        }

        /// Inserts public key into the `TestVerifyingKeyProvider`
        pub fn add_pk(
            &mut self,
            kid: CatalystId,
            pk: VerifyingKey,
        ) {
            self.verifying_key.insert(kid, pk);
        }
    }

    impl CatalystSignedDocumentProvider for TestCatalystProvider {
        async fn try_get_doc(
            &self,
            doc_ref: &DocumentRef,
        ) -> anyhow::Result<Option<CatalystSignedDocument>> {
            Ok(self.signed_doc.get(doc_ref).cloned())
        }

        async fn try_get_all(
            &self,
            id: catalyst_types::uuid::UuidV7,
        ) -> anyhow::Result<Vec<CatalystSignedDocument>> {
            Ok(self
                .signed_doc
                .iter()
                .filter(|(doc_ref, _)| doc_ref.id() == &id)
                .map(|(_, doc)| doc.clone())
                .collect())
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

    impl CatalystIdProvider for TestCatalystProvider {
        async fn try_get_registered_key(
            &self,
            kid: &CatalystId,
        ) -> anyhow::Result<Option<VerifyingKey>> {
            Ok(self.verifying_key.get(kid).copied())
        }
    }
}
