//! Implementation of the
//! `catalyst_signed_doc::validator::CatalystSignedDocumentValidationRule` trait for the
//! `Contest Delegation` document

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

use super::{get_delegations, get_delegator, get_payload};

/// `CatalystSignedDocumentValidationRule` implementation for Contest Delegation document.
#[derive(Debug)]
pub struct ContestDelegationRule;

impl CatalystSignedDocumentValidationRule for ContestDelegationRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let mut valid = true;

        valid &= get_delegator(doc, doc.report()).1;
        let (payload, is_payload_valid) = get_payload(doc, doc.report());
        valid &= is_payload_valid;

        valid &= get_delegations(doc, payload, provider, doc.report())?.1;

        Ok(valid)
    }
}
