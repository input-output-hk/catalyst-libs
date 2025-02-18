//! `content-type` rule type impl.

use catalyst_types::problem_report::ProblemReport;
use futures::{future::BoxFuture, FutureExt};

use crate::{
    metadata::ContentType, providers::CatalystSignedDocumentProvider, validator::ValidationRule,
    CatalystSignedDocument,
};

/// `content-type` field validation rule
pub(crate) struct ContentTypeRule {
    /// expected `content-type` field
    pub(crate) exp: ContentType,
}
impl<Provider> ValidationRule<Provider> for ContentTypeRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, _provider: &'a Provider,
        report: &'a ProblemReport,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if doc.doc_content_type() != self.exp {
                report.invalid_value(
                    "content-type",
                    doc.doc_content_type().to_string().as_str(),
                    self.exp.to_string().as_str(),
                    "Invalid Document content-type value",
                );
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
