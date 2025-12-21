//! `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.

use anyhow::{bail, ensure};
use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types::CONTEST_BALLOT, providers::Provider,
    validator::CatalystSignedDocumentValidationRule,
};

/// `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.
#[derive(Debug)]
pub struct ContestBallotRule {}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for ContestBallotRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        if doc.report().is_problematic() {
            bail!("Provided document is not valid {:?}", doc.report())
        }
        ensure!(
            doc.doc_type()? == &CONTEST_BALLOT,
            "Document must be Contest Ballot type"
        );

        todo!()
    }
}
