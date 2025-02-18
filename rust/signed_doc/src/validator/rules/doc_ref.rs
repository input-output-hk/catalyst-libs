//! `ref` rule type impl.

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};
use futures::{future::BoxFuture, FutureExt};

use crate::{
    providers::CatalystSignedDocumentProvider,
    validator::{utils::validate_provided_doc, ValidationRule},
    CatalystSignedDocument,
};

/// `ref` field validation rule
pub(crate) struct RefRule {
    /// expected `type` field of the referenced doc
    pub(crate) ref_type: Uuid,
    /// optional flag for the `ref` field
    pub(crate) optional: bool,
} 
impl<Provider> ValidationRule<Provider> for RefRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, provider: &'a Provider,
        report: &'a ProblemReport,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |proposal_doc: CatalystSignedDocument| -> bool {
                    if proposal_doc.doc_type().uuid() != self.ref_type {
                        report.invalid_value(
                            "ref",
                            proposal_doc.doc_type().to_string().as_str(),
                            self.ref_type.to_string().as_str(),
                            "Invalid referenced proposal document type",
                        );
                        return false;
                    }
                    true
                };
                return validate_provided_doc(&doc_ref, provider, report, ref_validator).await;
            } else if !self.optional {
                report.missing_field("ref", "Document must have a ref field");
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
