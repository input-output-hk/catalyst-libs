//! `reply` rule type impl.

use catalyst_types::uuid::UuidV4;

use super::doc_ref::referenced_doc_check;
use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `reply` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ReplyRule {
    /// Is 'reply' specified
    Specified {
        /// expected `type` field of the replied doc
        exp_reply_type: UuidV4,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'reply' is not specified
    NotSpecified,
}

impl ReplyRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_reply_type,
            optional,
        } = self
        {
            if let Some(reply) = doc.doc_meta().reply() {
                let reply_validator = |replied_doc: CatalystSignedDocument| {
                    if !referenced_doc_check(
                        &replied_doc,
                        exp_reply_type.uuid(),
                        "reply",
                        doc.report(),
                    ) {
                        return false;
                    }
                    let Some(doc_ref) = doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Document must have a ref field");
                        return false;
                    };

                    let Some(replied_doc_ref) = replied_doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Referenced document must have ref field");
                        return false;
                    };

                    if replied_doc_ref.id != doc_ref.id {
                        doc.report().invalid_value(
                                "reply",
                                doc_ref.id .to_string().as_str(),
                                replied_doc_ref.id.to_string().as_str(),
                                "Invalid referenced document. Document ID should aligned with the replied document.",
                            );
                        return false;
                    }

                    true
                };
                return validate_provided_doc(&reply, provider, doc.report(), reply_validator)
                    .await;
            } else if !optional {
                doc.report()
                    .missing_field("reply", "Document must have a reply field");
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(reply) = doc.doc_meta().reply() {
                doc.report().unknown_field(
                    "reply",
                    &reply.to_string(),
                    "Document does not expect to have a reply field",
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
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_reply_type = UuidV4::new();
        let common_ref_id = UuidV7::new();

        let valid_replied_doc_id = UuidV7::new();
        let another_type_replied_doc_id = UuidV7::new();
        let missing_ref_replied_doc_id = UuidV7::new();
        let missing_type_replied_doc_id = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string() },
                    "id": valid_replied_doc_id.to_string(),
                    "type": exp_reply_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string() },
                    "id": another_type_replied_doc_id.to_string(),
                    "type": UuidV4::new().to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `ref` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_ref_replied_doc_id.to_string(),
                    "type": exp_reply_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string() },
                    "id": missing_type_replied_doc_id.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ReplyRule::Specified {
            exp_reply_type,
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string() },
                "reply": { "id": valid_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `reply` field is missing, but its optional
        let rule = ReplyRule::Specified {
            exp_reply_type,
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `reply` field, but its required
        let rule = ReplyRule::Specified {
            exp_reply_type,
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string() },
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "reply": { "id": valid_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string() },
                "reply": { "id": another_type_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string() },
                "reply": { "id": missing_ref_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": common_ref_id.to_string() },
                "reply": { "id": missing_type_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // `ref` field does not align with the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": UuidV7::new().to_string() },
                "reply": { "id": valid_replied_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string() },
                "reply": {"id": UuidV7::new().to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn reply_rule_not_specified_test() {
        let rule = ReplyRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"reply": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
