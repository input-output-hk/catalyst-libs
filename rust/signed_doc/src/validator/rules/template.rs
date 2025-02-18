//! `template` rule type impl.

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};
use futures::{future::BoxFuture, FutureExt};

use crate::{
    metadata::ContentType,
    providers::CatalystSignedDocumentProvider,
    validator::{utils::validate_provided_doc, ValidationRule},
    CatalystSignedDocument,
};

/// `content-type` field validation rule
pub(crate) struct TemplateRule {
    /// expected `type` field of the template
    pub(crate) template_type: Uuid,
}
impl<Provider> ValidationRule<Provider> for TemplateRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, provider: &'a Provider,
        report: &'a ProblemReport,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            let Some(template_ref) = doc.doc_meta().template() else {
                report.missing_field("template", "Document must have a template field");
                return Ok(false);
            };

            let template_validator = |template_doc: CatalystSignedDocument| {
                if template_doc.doc_type().uuid() != self.template_type {
                    report.invalid_value(
                        "template",
                        template_doc.doc_type().to_string().as_str(),
                        self.template_type.to_string().as_str(),
                        "Invalid referenced template document type",
                    );
                    return false;
                }
                match doc.doc_content_type() {
                    ContentType::Json => json_schema_check(doc, &template_doc, report),
                    ContentType::Cbor => {
                        // TODO: not implemented yet
                        true
                    },
                }
            };
            validate_provided_doc(&template_ref, provider, report, template_validator).await
        }
        .boxed()
    }
}

/// Validate a provided `doc` against the `template` content's Json schema, assuming that
/// the `doc` content is JSON.
fn json_schema_check(
    doc: &CatalystSignedDocument, template_doc: &CatalystSignedDocument, report: &ProblemReport,
) -> bool {
    let Ok(template_json_schema) =
        serde_json::from_slice(template_doc.doc_content().decoded_bytes())
    else {
        report.functional_validation(
            "Template document content must be json encoded",
            "Invalid referenced template document content",
        );
        return false;
    };
    let Ok(schema_validator) = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(&template_json_schema)
    else {
        report.functional_validation(
            "Template document content must be Draft 7 JSON schema",
            "Invalid referenced template document content",
        );
        return false;
    };

    let Ok(doc_json) = serde_json::from_slice(doc.doc_content().decoded_bytes()) else {
        report.functional_validation(
            "Document content must be json encoded",
            "Invalid referenced template document content",
        );
        return false;
    };

    if schema_validator.validate(&doc_json).is_err() {
        report.functional_validation(
            "Proposal document content does not compliant with the template json schema",
            "Invalid Proposal document content",
        );
        return false;
    }
    true
}
