//! `template` rule type impl.

use std::fmt::Write;

use catalyst_types::uuid::UuidV4;

use super::doc_ref::referenced_doc_check;
use crate::{
    metadata::ContentType, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
};

/// Enum represents different content schemas, against which documents content would be
/// validated.
#[allow(dead_code)]
pub(crate) enum ContentSchema {
    /// Draft 7 JSON schema
    Json(jsonschema::Validator),
}

/// Document's content validation rule
pub(crate) enum ContentRule {
    /// Based on the 'template' field and loaded corresponding template document
    Templated {
        /// expected `type` field of the template
        exp_template_type: UuidV4,
    },
    /// Statically defined document's content schema.
    /// `template` field should not been specified
    #[allow(dead_code)]
    Static(ContentSchema),
    /// 'template' field is not specified
    #[allow(dead_code)]
    NotSpecified,
}

impl ContentRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Templated { exp_template_type } = self {
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
                    ContentType::Json => templated_json_schema_check(doc, &template_doc),
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
        if let Self::Static(content_schema) = self {
            if let Some(template) = doc.doc_meta().template() {
                doc.report().unknown_field(
                    "template",
                    &template.to_string(),
                    "Document does not expect to have a template field",
                );
                return Ok(false);
            }

            return Ok(content_schema_check(doc, content_schema));
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
fn templated_json_schema_check(
    doc: &CatalystSignedDocument, template_doc: &CatalystSignedDocument,
) -> bool {
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

    content_schema_check(doc, &ContentSchema::Json(schema_validator))
}

fn content_schema_check(doc: &CatalystSignedDocument, schema: &ContentSchema) -> bool {
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

    match schema {
        ContentSchema::Json(schema_validator) => {
            let schema_validation_errors =
                schema_validator
                    .iter_errors(&doc_json)
                    .fold(String::new(), |mut str, e| {
                        let _ = write!(str, "{{ {e} }}, ");
                        str
                    });

            if !schema_validation_errors.is_empty() {
                doc.report().functional_validation(
            &format!(
                "Proposal document content does not compliant with the json schema. [{schema_validation_errors}]"
            ),
            "Invalid Proposal document content",
        );
                return false;
            }
        },
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
    async fn content_rule_templated_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_template_type = UuidV4::new();
        let content_type = ContentType::Json;
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        let valid_template_doc_id = UuidV7::new();
        let valid_template_doc_ver = UuidV7::new();
        let another_type_template_doc_id = UuidV7::new();
        let another_type_template_doc_ver = UuidV7::new();
        let missing_type_template_doc_id = UuidV7::new();
        let missing_type_template_doc_ver = UuidV7::new();
        let missing_content_template_doc_id = UuidV7::new();
        let missing_content_template_doc_ver = UuidV7::new();
        let invalid_content_template_doc_id = UuidV7::new();
        let invalid_content_template_doc_ver = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": valid_template_doc_id.to_string(),
                    "ver": valid_template_doc_ver.to_string(),
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
                    "ver": another_type_template_doc_ver.to_string(),
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
                    "ver": missing_type_template_doc_ver.to_string(),
                }))
                .unwrap()
                .with_decoded_content(json_schema.clone())
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing content
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_content_template_doc_id.to_string(),
                    "ver": missing_content_template_doc_ver.to_string(),
                    "type": exp_template_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // invalid content, must be json encoded
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": invalid_content_template_doc_id.to_string(),
                    "ver": invalid_content_template_doc_ver.to_string(),
                    "type": exp_template_type.to_string()
                }))
                .unwrap()
                .with_decoded_content(vec![])
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ContentRule::Templated { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string(), "ver": valid_template_doc_ver.to_string() }
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
        let rule = ContentRule::Templated { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "template": {"id": valid_template_doc_id.to_string(), "ver": valid_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content
        let rule = ContentRule::Templated { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string(), "ver": valid_template_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded
        let rule = ContentRule::Templated { exp_template_type };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": valid_template_doc_id.to_string(), "ver": valid_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(vec![])
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": another_type_template_doc_id.to_string(), "ver": another_type_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_type_template_doc_id.to_string(), "ver": missing_type_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_content_template_doc_id.to_string(), "ver": missing_content_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": missing_content_template_doc_id.to_string(), "ver": missing_content_template_doc_ver.to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": content_type.to_string(),
                "template": {"id": UuidV7::new().to_string(), "ver": UuidV7::new().to_string() }
            }))
            .unwrap()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn content_rule_static_test() {
        let provider = TestCatalystSignedDocumentProvider::default();

        let json_schema = ContentSchema::Json(
            jsonschema::options()
                .with_draft(jsonschema::Draft::Draft7)
                .build(&serde_json::json!({}))
                .unwrap(),
        );
        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        // all correct
        let rule = ContentRule::Static(json_schema);
        let doc = Builder::new()
            .with_decoded_content(json_content.clone())
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing content
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded
        let doc = Builder::new().with_decoded_content(vec![]).build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // defined `template` field which should be absent
        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc =  Builder::new().with_decoded_content(json_content)
            .with_json_metadata(serde_json::json!({"template": {"id": ref_id.to_string(), "ver": ref_ver.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn template_rule_not_specified_test() {
        let rule = ContentRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // defined `template` field which should be absent
        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"template": {"id": ref_id.to_string(), "ver": ref_ver.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
