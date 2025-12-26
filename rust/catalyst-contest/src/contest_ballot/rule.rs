//! `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.
#[derive(Debug)]
pub struct ContestBallotRule {}

impl CatalystSignedDocumentValidationRule for ContestBallotRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        // TODO: FIXME:
        todo!()
    }
}
