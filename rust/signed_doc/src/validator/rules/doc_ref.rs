//! `ref` rule type impl.

use catalyst_types::uuid::UuidV4;

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `ref` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RefRule {
    /// Is 'ref' specified
    Specified {
        /// expected `type` field of the referenced doc
        exp_ref_type: UuidV4,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_ref_type,
            optional,
        } = self
        {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |ref_doc: CatalystSignedDocument| {
                    let Ok(ref_doc_type) = ref_doc.doc_type() else {
                        doc.report().missing_field(
                            "type",
                            "Missing type field for the referenced document",
                        );
                        return Ok(false);
                    };
                    if &ref_doc_type != exp_ref_type {
                        doc.report().invalid_value(
                            "ref",
                            ref_doc_type.to_string().as_str(),
                            exp_ref_type.to_string().as_str(),
                            "Invalid referenced document type",
                        );
                        return Ok(false);
                    }
                    Ok(true)
                };
                return validate_provided_doc(&doc_ref, provider, doc.report(), ref_validator)
                    .await;
            } else if !optional {
                doc.report()
                    .missing_field("ref", "Document must have a ref field");
                return Ok(false);
            }
        }
        if &Self::NotSpecified == self {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                doc.report().unknown_field(
                    "ref",
                    &doc_ref.to_string(),
                    "Document does not expect to have a ref field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::UuidV7;

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let doc_type = UuidV4::new();

        let rule = RefRule::Specified {
            exp_ref_type: doc_type,
            optional: true,
        };

        let provider = TestCatalystSignedDocumentProvider(|_| {
            Ok(Some(
                Builder::new()
                    .with_json_metadata(serde_json::json!({"type": doc_type.to_string()}))
                    .unwrap()
                    .build(),
            ))
        });

        // all correct
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `ref` field is missing, but its optional
        let rule = RefRule::Specified {
            exp_ref_type: doc_type,
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field, but its required
        let rule = RefRule::Specified {
            exp_ref_type: doc_type,
            optional: false,
        };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
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
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| Ok(Some(Builder::new().build())));
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| Ok(None));
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Provider returns an error
        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| anyhow::bail!("some error"));
        assert!(rule.check(&doc, &provider).await.is_err());
    }

    #[tokio::test]
    async fn ref_rule_not_specified_test() {
        let rule = RefRule::NotSpecified;

        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        let provider = TestCatalystSignedDocumentProvider(|_| anyhow::bail!("some error"));
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
