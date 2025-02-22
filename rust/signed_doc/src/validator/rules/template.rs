//! `template` rule type impl.

use catalyst_types::uuid::Uuid;

use crate::{
    CatalystSignedDocument, metadata::ContentType, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc,
};

/// `template` field validation rule
pub(crate) enum TemplateRule {
    /// Is 'template' specified
    Specified {
        /// expected `type` field of the template
        exp_template_type: Uuid,
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
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let Self::Specified { exp_template_type } = self {
            let Some(template_ref) = doc.doc_meta().template() else {
                doc.report()
                    .missing_field("template", "Document must have a template field");
                return Ok(false);
            };

            let template_validator = |template_doc: CatalystSignedDocument| {
                if &template_doc.doc_type()?.uuid() != exp_template_type {
                    doc.report().invalid_value(
                        "template",
                        template_doc.doc_type()?.to_string().as_str(),
                        exp_template_type.to_string().as_str(),
                        "Invalid referenced template document type",
                    );
                    return Ok(false);
                }
                match doc.doc_content_type()? {
                    ContentType::Json => json_schema_check(doc, &template_doc),
                    ContentType::Cbor => {
                        // TODO: not implemented yet
                        Ok(true)
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
fn json_schema_check(
    doc: &CatalystSignedDocument, template_doc: &CatalystSignedDocument,
) -> anyhow::Result<bool> {
    let Ok(template_json_schema) =
        serde_json::from_slice(template_doc.doc_content().decoded_bytes()?)
    else {
        doc.report().functional_validation(
            "Template document content must be json encoded",
            "Invalid referenced template document content",
        );
        return Ok(false);
    };
    let Ok(schema_validator) = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(&template_json_schema)
    else {
        doc.report().functional_validation(
            "Template document content must be Draft 7 JSON schema",
            "Invalid referenced template document content",
        );
        return Ok(false);
    };

    let Ok(doc_json) = serde_json::from_slice(doc.doc_content().decoded_bytes()?) else {
        doc.report().functional_validation(
            "Document content must be json encoded",
            "Invalid referenced template document content",
        );
        return Ok(false);
    };

    if schema_validator.validate(&doc_json).is_err() {
        doc.report().functional_validation(
            "Proposal document content does not compliant with the template json schema",
            "Invalid Proposal document content",
        );
        return Ok(false);
    }
    Ok(true)
}
