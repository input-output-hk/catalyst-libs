//! `section` rule type impl.

use futures::{future::BoxFuture, FutureExt};

use crate::{
    providers::CatalystSignedDocumentProvider, validator::ValidationRule, CatalystSignedDocument,
};

/// `section` field validation rule
pub(crate) struct SectionRule {
    /// optional flag for the `section` field
    pub(crate) optional: bool,
}
impl<Provider> ValidationRule<Provider> for SectionRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, _provider: &'a Provider,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if doc.doc_meta().section().is_none() && !self.optional {
                doc.report()
                    .missing_field("section", "Document must have a section field");
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
