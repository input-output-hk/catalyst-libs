//! `content-encoding` rule type impl.

use catalyst_types::problem_report::ProblemReport;
use futures::{future::BoxFuture, FutureExt};

use crate::{
    metadata::ContentEncoding, providers::CatalystSignedDocumentProvider,
    validator::ValidationRule, CatalystSignedDocument,
};

/// `content-encoding` field validation rule
pub(crate) struct ContentEncodingRule {
    /// expected `content-encoding` field
    pub(crate) exp: ContentEncoding,
    /// optional flag for the `content-encoding` field
    pub(crate) optional: bool,
}
impl<Provider> ValidationRule<Provider> for ContentEncodingRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, _provider: &'a Provider,
        report: &'a ProblemReport,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if let Some(content_encoding) = doc.doc_content_encoding() {
                if content_encoding != self.exp {
                    report.invalid_value(
                        "content-encoding",
                        content_encoding.to_string().as_str(),
                        self.exp.to_string().as_str(),
                        "Invalid Document content-encoding value",
                    );
                    return Ok(false);
                }
            } else if !self.optional {
                report.missing_field(
                    "content-encoding",
                    "Document must have a content-encoding field",
                );
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
