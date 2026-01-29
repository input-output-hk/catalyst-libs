//! A special tally providers trait definitions

use catalyst_signed_doc::{catalyst_id::CatalystId, providers::CatalystSignedDocumentProvider};

/// A Special Tally Provider, which is a combination of `VotingPowerProvider` and
/// `CatalystSignedDocumentProvider`
pub trait TallyProvider: VotingPowerProvider + CatalystSignedDocumentProvider {}

impl<T: VotingPowerProvider + CatalystSignedDocumentProvider> TallyProvider for T {}

/// A Voter's voting power provider
pub trait VotingPowerProvider {
    /// Try to get a voting power value by the provided user's `CatalystId`.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    fn try_get_voting_power(
        &self,
        kid: &CatalystId,
    ) -> anyhow::Result<u64>;
}

#[cfg(test)]
pub(crate) mod tests {
    //! Simple providers implementation just for the testing purposes

    use std::{collections::HashMap};

    use anyhow::Context;
    use catalyst_signed_doc::providers::tests::TestCatalystProvider;

    use super::*;

    #[derive(Default)]
    pub struct TestTallyProvider {
        pub voters: HashMap<CatalystId, u64>,
        pub p: TestCatalystProvider,
    }


    impl VotingPowerProvider for TestTallyProvider {
        fn try_get_voting_power(
            &self,
            kid: &catalyst_signed_doc::catalyst_id::CatalystId,
        ) -> anyhow::Result<u64> {
            self.voters
                .get(kid)
                .copied()
                .context("Cannot find a voter's voting power")
        }
    }

    impl CatalystSignedDocumentProvider for TestTallyProvider {
        fn try_get_doc(
            &self,
            doc_ref: &catalyst_signed_doc::DocumentRef,
        ) -> anyhow::Result<Option<catalyst_signed_doc::CatalystSignedDocument>> {
            self.p.try_get_doc(doc_ref)
        }

        fn try_get_first_doc(
            &self,
            id: catalyst_signed_doc::uuid::UuidV7,
        ) -> anyhow::Result<Option<catalyst_signed_doc::CatalystSignedDocument>> {
            self.p.try_get_first_doc(id)
        }

        fn try_get_last_doc(
            &self,
            id: catalyst_signed_doc::uuid::UuidV7,
        ) -> anyhow::Result<Option<catalyst_signed_doc::CatalystSignedDocument>> {
            self.p.try_get_last_doc(id)
        }

        fn try_search_docs(
            &self,
            query: &catalyst_signed_doc::providers::CatalystSignedDocumentSearchQuery,
        ) -> anyhow::Result<Vec<catalyst_signed_doc::CatalystSignedDocument>> {
            self.p.try_search_docs(query)
        }
    }
}
