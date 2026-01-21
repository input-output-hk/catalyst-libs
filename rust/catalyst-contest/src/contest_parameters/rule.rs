//! Implementation of the
//! `catalyst_signed_doc::validator::CatalystSignedDocumentValidationRule` trait for the
//! `Contest Parameters` document

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

use super::get_payload;

/// `CatalystSignedDocumentValidationRule` implementation for Contest Parameters document.
#[derive(Debug)]
pub struct ContestParametersRule;

impl CatalystSignedDocumentValidationRule for ContestParametersRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        get_payload(doc, doc.report());

        Ok(!doc.report().is_problematic())
    }
}
