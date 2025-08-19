//! `template` rule type impl.

use std::fmt::Write;

use super::doc_ref::referenced_doc_check;
use crate::{
    metadata::ContentType,
    providers::CatalystSignedDocumentProvider,
    validator::{json_schema, utils::validate_doc_refs},
    CatalystSignedDocument, DocType,
};

/// Enum represents different content schemas, against which documents content would be
/// validated.
pub(crate) enum ContentSchema {
    /// Draft 7 JSON schema
    Json(json_schema::JsonSchema),
}

/// Document's content validation rule
pub(crate) enum ContentRule {
    /// Based on the 'template' field and loaded corresponding template document
    Templated {
        /// expected `type` field of the template
        exp_template_type: DocType,
    },
    /// Statically defined document's content schema.
    /// `template` field should not been specified
    Static(ContentSchema),
    /// 'template' field is not specified
    #[allow(dead_code)]
    NotSpecified,
}

impl ContentRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let context = "Content/Template rule check";
        if let Self::Templated { exp_template_type } = self {
            let Some(template_ref) = doc.doc_meta().template() else {
                doc.report()
                    .missing_field("template", &format!("{context}, doc"));
                return Ok(false);
            };
            let template_validator = |template_doc: CatalystSignedDocument| {
                if !referenced_doc_check(
                    &template_doc,
                    std::slice::from_ref(exp_template_type),
                    "template",
                    doc.report(),
                ) {
                    return false;
                }
                let Ok(template_content_type) = template_doc.doc_content_type() else {
                    doc.report().missing_field(
                        "content-type",
                        &format!("{context}, referenced document must have a content-type field"),
                    );
                    return false;
                };
                match template_content_type {
                    ContentType::Json => templated_json_schema_check(doc, &template_doc),
                    ContentType::Cddl
                    | ContentType::Cbor
                    | ContentType::JsonSchema
                    | ContentType::Css
                    | ContentType::CssHandlebars
                    | ContentType::Html
                    | ContentType::HtmlHandlebars
                    | ContentType::Markdown
                    | ContentType::MarkdownHandlebars
                    | ContentType::Plain
                    | ContentType::PlainHandlebars => {
                        // TODO: not implemented yet
                        true
                    },
                }
            };
            return validate_doc_refs(template_ref, provider, doc.report(), template_validator)
                .await;
        }
        if let Self::Static(content_schema) = self {
            if let Some(template) = doc.doc_meta().template() {
                doc.report().unknown_field(
                    "template",
                    &template.to_string(),
                    &format!("{context} Static, Document does not expect to have a template field",)
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
                    &format!("{context} Not Specified, Document does not expect to have a template field",)
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
    doc: &CatalystSignedDocument,
    template_doc: &CatalystSignedDocument,
) -> bool {
    let Ok(template_content) = template_doc.decoded_content() else {
        doc.report().functional_validation(
            "Invalid document content, cannot get decoded bytes",
            "Cannot get a referenced template document content during the templated validation",
        );
        return false;
    };
    let Ok(template_json_schema) = serde_json::from_slice(&template_content) else {
        doc.report().functional_validation(
            "Template document content must be json encoded",
            "Invalid referenced template document content",
        );
        return false;
    };
    let Ok(schema) = json_schema::JsonSchema::try_from(&template_json_schema) else {
        doc.report().functional_validation(
            "Template document content must be Draft 7 JSON schema",
            "Invalid referenced template document content",
        );
        return false;
    };

    content_schema_check(doc, &ContentSchema::Json(schema))
}

/// Validating the document's content against the provided schema
fn content_schema_check(
    doc: &CatalystSignedDocument,
    schema: &ContentSchema,
) -> bool {
    let Ok(doc_content) = doc.decoded_content() else {
        doc.report().functional_validation(
            "Invalid Document content, cannot get decoded bytes",
            "Cannot get a document content during the templated validation",
        );
        return false;
    };
    if doc_content.is_empty() {
        doc.report()
            .missing_field("payload", "Document must have a content");
        return false;
    }
    let Ok(doc_json) = serde_json::from_slice(&doc_content) else {
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
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, DocLocator, DocumentRef,
    };

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn content_rule_templated_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_template_type = UuidV4::new();
        let content_type = ContentType::Json;
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        let valid_template_doc_id = UuidV7::new();
        let another_type_template_doc_id = UuidV7::new();
        let missing_type_template_doc_id = UuidV7::new();
        let missing_content_type_template_doc_id = UuidV7::new();
        let missing_content_template_doc_id = UuidV7::new();
        let invalid_content_template_doc_id = UuidV7::new();

        // Prepare provider documents
        {
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(valid_template_doc_id))
                .with_metadata_field(SupportedField::Ver(valid_template_doc_id))
                .with_metadata_field(SupportedField::Type(exp_template_type.into()))
                .with_metadata_field(SupportedField::ContentType(content_type))
                .with_content(json_schema.clone())
                .build();
            provider.add_document(None, &doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(another_type_template_doc_id))
                .with_metadata_field(SupportedField::Ver(another_type_template_doc_id))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .with_metadata_field(SupportedField::ContentType(content_type))
                .with_content(json_schema.clone())
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_type_template_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_type_template_doc_id))
                .with_metadata_field(SupportedField::ContentType(content_type))
                .with_content(json_schema.clone())
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            // missing `content-type` field in the referenced document
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_content_type_template_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_content_type_template_doc_id))
                .with_metadata_field(SupportedField::Type(exp_template_type.into()))
                .with_content(json_schema.clone())
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            // missing content
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_content_template_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_content_template_doc_id))
                .with_metadata_field(SupportedField::Type(exp_template_type.into()))
                .with_metadata_field(SupportedField::ContentType(content_type))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            // invalid content, must be json encoded
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(invalid_content_template_doc_id))
                .with_metadata_field(SupportedField::Ver(invalid_content_template_doc_id))
                .with_metadata_field(SupportedField::Type(exp_template_type.into()))
                .with_metadata_field(SupportedField::ContentType(content_type))
                .with_content(vec![])
                .build();
            provider.add_document(None, &ref_doc).unwrap();
        }

        // Create a document where `templates` field is required and referencing a valid document
        // in provider. Using doc ref of new implementation.
        let rule = ContentRule::Templated {
            exp_template_type: exp_template_type.into(),
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    valid_template_doc_id,
                    valid_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `template` field, but its required
        let doc = Builder::new().with_content(json_content.clone()).build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content
        let rule = ContentRule::Templated {
            exp_template_type: exp_template_type.into(),
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    valid_template_doc_id,
                    valid_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded
        let rule = ContentRule::Templated {
            exp_template_type: exp_template_type.into(),
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    valid_template_doc_id,
                    valid_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(vec![1, 2, 3])
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    another_type_template_doc_id,
                    another_type_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    missing_type_template_doc_id,
                    missing_type_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `content-type` field in the referenced doc
        let rule = ContentRule::Templated {
            exp_template_type: exp_template_type.into(),
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    missing_content_type_template_doc_id,
                    missing_content_type_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing content in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    missing_content_template_doc_id,
                    missing_content_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    invalid_content_template_doc_id,
                    invalid_content_template_doc_id,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .with_content(json_content.clone())
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn content_rule_static_test() {
        let provider = TestCatalystSignedDocumentProvider::default();
        let schema = json_schema::JsonSchema::try_from(&serde_json::json!({})).unwrap();
        let content_schema = ContentSchema::Json(schema);
        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        // all correct
        let rule = ContentRule::Static(content_schema);
        let doc = Builder::new().with_content(json_content.clone()).build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing content
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // content not a json encoded
        let doc = Builder::new().with_content(vec![1, 2, 3]).build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // defined `template` field which should be absent
        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
            ))
            .with_content(json_content)
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
            .with_metadata_field(SupportedField::Template(
                vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
