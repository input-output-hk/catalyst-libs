//! `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

use crate::contest_ballot::ballot::{check_parameters, check_proof, payload};

/// `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.
#[derive(Debug)]
pub struct ContestBallotRule {}

impl CatalystSignedDocumentValidationRule for ContestBallotRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let payload = payload(doc, doc.report());
        let params = check_parameters(doc, provider, doc.report())?;
        if let (Some(payload), Some(params)) = &(payload, params) {
            check_proof(payload, params, doc.report())?;
        }

        Ok(!doc.report().is_problematic())
    }
}
