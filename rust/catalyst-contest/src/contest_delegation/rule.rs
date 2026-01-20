//! Implementation of the
//! `catalyst_signed_doc::validator::CatalystSignedDocumentValidationRule` trait for the
//! `Contest Delegation` document

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

use super::{contest_parameters_checks, get_delegations, get_delegator, get_payload};

/// `CatalystSignedDocumentValidationRule` implementation for Contest Delegation document.
#[derive(Debug)]
pub struct ContestDelegationRule;

impl CatalystSignedDocumentValidationRule for ContestDelegationRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        get_delegator(doc, doc.report());
        let payload = get_payload(doc, doc.report());
        contest_parameters_checks(doc, provider, doc.report())?;
        get_delegations(doc, payload, provider, doc.report())?;

        Ok(!doc.report().is_problematic())
    }
}
