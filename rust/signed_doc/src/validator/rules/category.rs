//! `content-type` rule type impl.

use super::doc_ref::referenced_doc_check;
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
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified { optional } = self {
            if let Some(category) = &doc.doc_meta().category_id() {
                let category_validator = |category_doc: CatalystSignedDocument| {
                    Ok(referenced_doc_check(
                        &category_doc,
                        CATEGORY_DOCUMENT_UUID_TYPE,
                        "category_id",
                        doc.report(),
                    ))
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

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[tokio::test]
    async fn category_rule_specified_test() {
        let rule = CategoryRule::Specified { optional: true };

        let provider = TestCatalystSignedDocumentProvider(|_| {
            Ok(Some(
                Builder::new()
                    .with_json_metadata(
                        serde_json::json!({"type": CATEGORY_DOCUMENT_UUID_TYPE.to_string()}),
                    )
                    .unwrap()
                    .build(),
            ))
        });

        // all correct
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"category_id": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `category_id` field is missing, but its optional
        let rule = CategoryRule::Specified { optional: true };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `category_id` field, but its required
        let rule = CategoryRule::Specified { optional: false };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"category_id": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| {
            let another_doc_type = UuidV4::new();
            Ok(Some(
                Builder::new()
                    .with_json_metadata(serde_json::json!({"type": another_doc_type.to_string()}))
                    .unwrap()
                    .build(),
            ))
        });
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"category_id": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| Ok(Some(Builder::new().build())));
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"category_id": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| Ok(None));
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn category_rule_not_specified_test() {
        let rule = CategoryRule::NotSpecified;

        let doc = Builder::new().build();
        let provider = TestCatalystSignedDocumentProvider(|_| anyhow::bail!("some error"));
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"category_id": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| anyhow::bail!("some error"));
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
