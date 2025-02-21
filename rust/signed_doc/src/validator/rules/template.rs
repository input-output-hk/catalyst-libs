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

    if schema_validator.validate(&doc_json).is_err() {
        doc.report().functional_validation(
            "Proposal document content does not compliant with the template json schema",
            "Invalid Proposal document content",
        );
        return false;
    }
    true
}
