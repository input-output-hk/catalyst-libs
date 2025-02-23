//! `template` rule type impl.

use catalyst_types::uuid::UuidV4;

use super::doc_ref::referenced_doc_check;
use crate::{
    metadata::ContentType, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
};

/// `template` field validation rule
pub(crate) enum TemplateRule {
    /// Is 'template' specified
    Specified {
        /// expected `type` field of the template
        exp_template_type: UuidV4,
    },
    /// 'template' is not specified
    #[allow(dead_code)]
    NotSpecified,
}

impl TemplateRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified { exp_template_type } = self {
            let Some(template_ref) = doc.doc_meta().template() else {
                doc.report()
                    .missing_field("template", "Document must have a template field");
                return Ok(false);
            };

            let template_validator = |template_doc: CatalystSignedDocument| {
                if !referenced_doc_check(
                    &template_doc,
                    exp_template_type.uuid(),
                    "template",
                    doc.report(),
                ) {
                    return false;
                }

                let Ok(doc_content_type) = doc.doc_content_type() else {
                    doc.report()
                        .missing_field("content-type", "Document must have a content-type field");
                    return false;
                };
                match doc_content_type {
                    ContentType::Json => json_schema_check(doc, &template_doc),
                    ContentType::Cbor => {
                        // TODO: not implemented yet
                        true
                    },
                }
            };
            return validate_provided_doc(
                &template_ref,
                provider,
                doc.report(),
                template_validator,
            )
            .await;
        }
        if let Self::NotSpecified = self {
            if let Some(template) = doc.doc_meta().template() {
                doc.report().unknown_field(
                    "template",
                    &template.to_string(),
                    "Document does not expect to have a template field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Validate a provided `doc` against the `template` content's Json schema, assuming that
/// the `doc` content is JSON.
fn json_schema_check(doc: &CatalystSignedDocument, template_doc: &CatalystSignedDocument) -> bool {
    let Ok(template_content) = template_doc.doc_content().decoded_bytes() else {
        doc.report().missing_field(
            "payload",
            "Referenced template document must have a content",
        );
        return false;
    };
    let Ok(template_json_schema) = serde_json::from_slice(template_content) else {
        doc.report().functional_validation(
            "Template document content must be json encoded",
            "Invalid referenced template document content",
        );
        return false;
    };
    let Ok(schema_validator) = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(&template_json_schema)
    else {
        doc.report().functional_validation(
            "Template document content must be Draft 7 JSON schema",
            "Invalid referenced template document content",
        );
        return false;
    };

    let Ok(doc_content) = doc.doc_content().decoded_bytes() else {
        doc.report()
            .missing_field("payload", "Document must have a content");
        return false;
    };
    let Ok(doc_json) = serde_json::from_slice(doc_content) else {
        doc.report().functional_validation(
            "Document content must be json encoded",
            "Invalid referenced template document content",
        );
        return false;
    };

    if let Err(e) = schema_validator.validate(&doc_json) {
        doc.report().functional_validation(
            &format!(
                "Proposal document content does not compliant with the template json schema. {e}"
            ),
            "Invalid Proposal document content",
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::UuidV7;

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_template_type = UuidV4::new();
        let content_type = ContentType::Json;
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        let valid_template_doc_id = UuidV7::new();
        let another_type_template_doc_id = UuidV7::new();
        let missing_type_template_doc_id = UuidV7::new();
        let missing_content_template_doc_id = UuidV7::new();
        let invalid_content_template_doc_id = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": valid_template_doc_id.to_string(),
                    "type": exp_template_type.to_string()
                }))
                .unwrap()
                .with_decoded_content(json_schema.clone())
                .build();
            provider.add_document(ref_doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": another_type_template_doc_id.to_string(),
                    "type": UuidV4::new().to_string()
                }))
                .unwrap()
                .with_decoded_content(json_schema.clone())
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_type_template_doc_id.to_string(),
                }))
                .unwrap()
                .with_decoded_content(json_schema.clone())
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing content
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_content_template_doc_id.to_string(),
                    "type": exp_template_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // invalid content, must be json encoded
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": invalid_content_template_doc_id.to_string(),
                    "type": exp_template_type.to_string()
                }))
                .unwrap()
                .with_decoded_content(vec![])
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = TemplateRule::Specified { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `template` field, but its required
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `content-type` field
        let rule = TemplateRule::Specified { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "template": {"id": valid_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content
        let rule = TemplateRule::Specified { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded
        let rule = TemplateRule::Specified { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(vec![])
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": another_type_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_type_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_content_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_content_template_doc_id.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": UuidV7::new().to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn template_rule_not_specified_test() {
        let rule: TemplateRule = TemplateRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"template": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
