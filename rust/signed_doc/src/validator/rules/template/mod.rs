//! `template` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{
    DocSpecs, DocumentName, is_required::IsRequired, metadata::template::Template,
};
use catalyst_types::json_schema::JsonSchema;

use crate::{
    CatalystSignedDocument, ContentType, DocType,
    providers::Provider,
    validator::{
        CatalystSignedDocumentValidationRule,
        rules::{doc_ref::doc_refs_check, utils::content_json_schema_check},
    },
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

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for TemplateRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider).await
    }
}

impl TemplateRule {
    /// Generating `TemplateRule` from specs
    pub(crate) fn new(
        docs: &DocSpecs,
        spec: &Template,
    ) -> anyhow::Result<Self> {
        if let IsRequired::Excluded = spec.required {
            anyhow::ensure!(
                spec.doc_type.is_empty() && !spec.multiple,
                "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'template'  metadata definition"
            );
            return Ok(Self::NotSpecified);
        }

        anyhow::ensure!(
            !spec.multiple,
            "'multiple' field should be only set to false for the required 'template' metadata definition"
        );
        anyhow::ensure!(
            spec.required != IsRequired::Optional,
            "'required' field cannot been 'optional' for 'template' metadata definition"
        );

        let doc_name = &<&[DocumentName; 1]>::try_from(spec.doc_type.as_slice()).map_err(|_| anyhow::anyhow!("'type' field should exists and has only one entry for the required 'template' metadata definition"))?[0];
        let docs_spec = docs.get(doc_name).ok_or(anyhow::anyhow!(
            "cannot find a document definition {doc_name}"
        ))?;
        let allowed_type = docs_spec.doc_type.as_str().parse()?;

        Ok(Self::Specified { allowed_type })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
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
                    ContentType::SchemaJson => templated_json_schema_check(doc, template_doc),
                    ContentType::Json
                    | ContentType::Cddl
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
                        false
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
        if let Self::NotSpecified = self
            && let Some(template) = doc.doc_meta().template()
        {
            doc.report().unknown_field(
                "template",
                &template.to_string(),
                &format!(
                    "{context} Not Specified, Document does not expect to have a template field",
                ),
            );
            return Ok(false);
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
