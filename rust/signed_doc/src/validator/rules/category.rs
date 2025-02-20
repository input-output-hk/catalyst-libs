//! `content-type` rule type impl.

use crate::{
    doc_types::CATEGORY_DOCUMENT_UUID_TYPE, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
};

/// `category_id` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CategoryRule {
    /// Is `category_id` specified
    Specified {
        /// optional flag for the `category_id` field
        optional: bool,
    },
    /// `category_id` is not specified
    NotSpecified,
}

impl CategoryRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let Self::Specified { optional } = self {
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
            } else if !optional {
                doc.report()
                    .missing_field("category_id", "Document must have a category field");
                return Ok(false);
            }
        }
        if &Self::NotSpecified == self {
            if let Some(category) = doc.doc_meta().category_id() {
                doc.report().unknown_field(
                    "category_id",
                    &category.to_string(),
                    "Document does not expect to have a category field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
