//! `template` rule type impl.

use std::collections::HashMap;

use catalyst_signed_doc_spec::{
    is_required::IsRequired, metadata::template::Template, DocSpec, DocumentName,
};

use crate::{
    providers::CatalystSignedDocumentProvider,
    validator::{
        json_schema::JsonSchema,
        rules::{doc_ref::doc_refs_check, utils::content_json_schema_check},
    },
    CatalystSignedDocument, ContentType, DocType,
};

/// `reply` field validation rule
#[derive(Debug)]
pub(crate) enum TemplateRule {
    /// Is 'template' specified
    Specified {
        /// allowed `type` field of the template
        allowed_type: DocType,
    },
    /// 'template' field is not specified
    NotSpecified,
}

impl TemplateRule {
    /// Generating `TemplateRule` from specs
    pub(crate) fn new(
        docs: &HashMap<DocumentName, DocSpec>,
        spec: &Template,
    ) -> anyhow::Result<Self> {
        if let IsRequired::Excluded = spec.required {
            anyhow::ensure!(
                spec.doc_type.is_empty() && spec.multiple.is_none(),
                "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'template'  metadata definition"
            );
            return Ok(Self::NotSpecified);
        }

        anyhow::ensure!(
            spec.multiple.is_some_and(|v| !v),
            "'multiple' field should be only set to false for the required 'template' metadata definition"
        );
        anyhow::ensure!(
            spec.required != IsRequired::Optional,
            "'required' field cannot been 'optional' for 'template' metadata definition"
        );

        let doc_name = &<&[DocumentName; 1]>::try_from(spec.doc_type.as_slice()).map_err(|_| anyhow::anyhow!("'type' field should exists and has only one entry for the required 'template' metadata definition"))?[0];
        let docs_spec = docs.get(&doc_name).ok_or(anyhow::anyhow!(
            "cannot find a document definition {doc_name}"
        ))?;
        let allowed_type = docs_spec.doc_type.as_str().parse()?;

        Ok(Self::Specified { allowed_type })
    }

    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let context = "Template rule check";

        if let Self::Specified { allowed_type } = self {
            let Some(template_ref) = doc.doc_meta().template() else {
                doc.report()
                    .missing_field("template", &format!("{context}, doc"));
                return Ok(false);
            };

            let template_validator = |template_doc: &CatalystSignedDocument| {
                let Some(template_content_type) = template_doc.doc_content_type() else {
                    doc.report().missing_field(
                        "content-type",
                        &format!("{context}, referenced document must have a content-type field"),
                    );
                    return false;
                };
                match template_content_type {
                    ContentType::Json | ContentType::SchemaJson => {
                        templated_json_schema_check(doc, template_doc)
                    },
                    ContentType::Cddl
                    | ContentType::Cbor
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

            return doc_refs_check(
                template_ref,
                std::slice::from_ref(allowed_type),
                false,
                "template",
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
    let Ok(schema) = JsonSchema::try_from(&template_json_schema) else {
        doc.report().functional_validation(
            "Template document content must be Draft 7 JSON schema",
            "Invalid referenced template document content",
        );
        return false;
    };

    content_json_schema_check(doc, &schema)
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        DocLocator, DocumentRef,
    };

    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => true
        ;
        "content is complied with the referenced template json schema"
    )]
    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "missing template field"
    )]
    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .build()
        }
        => false
        ;
        "missing content"
    )]
    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(vec![1, 2, 3,])
                .build()
        }
        => false
        ;
        "content is not valid JSON"
    )]
    #[test_case(
        |_, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "wrong 'type' in the referenced template document"
    )]
    #[test_case(
        |_, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "missing 'type' field in the referenced template document"
    )]
    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "missing 'content-type' field in the referenced template document'"
    )]
    #[test_case(
        |allowed_type, provider| {
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "missing content in the referenced template document"
    )]
    #[test_case(
        |allowed_type, provider| {
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(vec![1,2 ,3])
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "content is not a JSON schema in the referenced template document"
    )]
    #[test_case(
        |_, _| {
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "referencing to unknown document"
    )]
    #[tokio::test]
    async fn template_specified_test(
        doc_gen: impl FnOnce(DocType, &mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystProvider::default();

        let allowed_type: DocType = UuidV4::new().into();

        let doc = doc_gen(allowed_type.clone(), &mut provider);

        TemplateRule::Specified { allowed_type }
            .check(&doc, &provider)
            .await
            .unwrap()
    }

    #[test_case(
        |_, _| {
            Builder::new()
                .build()
        }
        => true
        ;
        "missing 'template' field"
    )]
    #[test_case(
        |allowed_type, provider| {
            let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
            let template_ref = DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            );
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(*template_ref.id()))
                .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
                .with_metadata_field(SupportedField::Type(allowed_type))
                .with_metadata_field(SupportedField::ContentType(ContentType::Json))
                .with_content(json_schema)
                .build();
            provider.add_document(None, &doc).unwrap();

            let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![template_ref].into(),
                ))
                .with_content(json_content)
                .build()
        }
        => false
        ;
        "content is complied with the referenced template json schema for non specified 'template' field"
    )]
    #[tokio::test]
    async fn reply_rule_not_specified_test(
        doc_gen: impl FnOnce(DocType, &mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let allowed_type: DocType = UuidV4::new().into();
        let mut provider = TestCatalystProvider::default();

        let doc = doc_gen(allowed_type, &mut provider);
        TemplateRule::NotSpecified
            .check(&doc, &provider)
            .await
            .unwrap()
    }
}
