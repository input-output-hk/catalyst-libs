//! `content-type` rule type impl.

use crate::{
    doc_types::CATEGORY_DOCUMENT_UUID_TYPE, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
};

/// `category_id` field validation rule
pub(crate) struct CategoryRule {
    /// optional flag for the `category_id` field
    pub(crate) optional: bool,
}

impl CategoryRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
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
}
