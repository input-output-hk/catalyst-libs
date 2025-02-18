//! `content-type` rule type impl.

use futures::{future::BoxFuture, FutureExt};

use crate::{
    doc_types::CATEGORY_DOCUMENT_UUID_TYPE,
    providers::CatalystSignedDocumentProvider,
    validator::{utils::validate_provided_doc, ValidationRule},
    CatalystSignedDocument,
};

/// `category_id` field validation rule
pub(crate) struct CategoryRule {
    /// optional flag for the `category_id` field
    pub(crate) optional: bool,
}
impl<Provider> ValidationRule<Provider> for CategoryRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, provider: &'a Provider,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if let Some(category) = &doc.doc_meta().category_id() {
                let category_validator = |category_doc: CatalystSignedDocument| {
                    if category_doc.doc_type()?.uuid() != CATEGORY_DOCUMENT_UUID_TYPE {
                        doc.report().invalid_value(
                            "category_id",
                            category_doc.doc_type()?.to_string().as_str(),
                            CATEGORY_DOCUMENT_UUID_TYPE.to_string().as_str(),
                            "Invalid referenced category document type",
                        );
                        return Ok(false);
                    }
                    Ok(true)
                };
                return validate_provided_doc(category, provider, doc.report(), category_validator)
                    .await;
            } else if !self.optional {
                doc.report()
                    .missing_field("category_id", "Document must have a category field");
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
