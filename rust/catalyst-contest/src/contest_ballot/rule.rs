//! `CatalystSignedDocumentValidationRule` implementation for `ContentBallotPayload`.

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

use crate::contest_ballot::{check_choices, check_parameters, payload};

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
        if let Some(params) = &params {
            check_choices(&payload, params, doc.report())?;
        }

        Ok(!doc.report().is_problematic())
    }
}
