//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

#![allow(dead_code)]

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};

use super::{CATEGORY_DOCUMENT_UUID_TYPE, PROPOSAL_TEMPLATE_UUID_TYPE};
use crate::{
    metadata::{ContentEncoding, ContentType},
    providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc,
    CatalystSignedDocument, DocumentRef,
};

/// Proposal document `UuidV4` type.
pub const PROPOSAL_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);

/// Proposal Document struct
pub struct ProposalDocument {
    /// `template` field
    template: DocumentRef,
    /// `category_id` field
    category: Option<DocumentRef>,
    /// Proposal content
    content: serde_json::Value,
}

/// `type` field validation
fn type_stateless_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_type().uuid() != PROPOSAL_DOCUMENT_UUID_TYPE {
        report.invalid_value(
            "type",
            doc.doc_type().to_string().as_str(),
            PROPOSAL_DOCUMENT_UUID_TYPE.to_string().as_str(),
            "Invalid Proposal Document type UUID value",
        );
        return false;
    }
    true
}

/// `content-type` validation
fn content_type_stateless_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_content_type() != ContentType::Json {
        report.invalid_value(
            "content-type",
            doc.doc_content_type().to_string().as_str(),
            ContentType::Json.to_string().as_str(),
            "Invalid Proposal Document content-type value",
        );
        return false;
    }
    true
}

/// `content-encoding` validation
fn content_encoding_stateless_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if let Some(content_encoding) = doc.doc_content_encoding() {
        if content_encoding != ContentEncoding::Brotli {
            report.invalid_value(
                "content-encoding",
                content_encoding.to_string().as_str(),
                ContentEncoding::Brotli.to_string().as_str(),
                "Invalid Proposal Document content-encoding value",
            );
            return false;
        }
    } else {
        report.missing_field(
            "content-encoding",
            "Proposal Document must have a content-encoding field",
        );
        return false;
    }
    true
}

/// `template` validation
fn template_stateless_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_meta().template().is_none() {
        report.missing_field("template", "Proposal Document must have a template field");
        return false;
    }
    true
}

/// `template` statefull validation
async fn template_statefull_check<Provider>(
    doc: &ProposalDocument, provider: &Provider, report: &ProblemReport,
) -> anyhow::Result<bool>
where Provider: 'static + CatalystSignedDocumentProvider {
    let template_validator = |template_doc: CatalystSignedDocument| {
        if template_doc.doc_type().uuid() != PROPOSAL_TEMPLATE_UUID_TYPE {
            report.invalid_value(
                "template",
                template_doc.doc_type().to_string().as_str(),
                PROPOSAL_TEMPLATE_UUID_TYPE.to_string().as_str(),
                "Invalid referenced template document type",
            );
            return false;
        }
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

        if schema_validator.validate(&doc.content).is_err() {
            report.functional_validation(
                "Proposal document content does not compliant with the template json schema",
                "Invalid Proposal document content",
            );
            return false;
        }
        true
    };
    validate_provided_doc(
        &doc.template,
        "Proposal Template",
        provider,
        report,
        template_validator,
    )
    .await
}

/// `category_id` statefull validation
async fn category_statefull_check<Provider>(
    doc: &ProposalDocument, provider: &Provider, report: &ProblemReport,
) -> anyhow::Result<bool>
where Provider: 'static + CatalystSignedDocumentProvider {
    if let Some(category) = &doc.category {
        let category_validator = |category_doc: CatalystSignedDocument| -> bool {
            if category_doc.doc_type().uuid() != CATEGORY_DOCUMENT_UUID_TYPE {
                report.invalid_value(
                    "category_id",
                    category_doc.doc_type().to_string().as_str(),
                    CATEGORY_DOCUMENT_UUID_TYPE.to_string().as_str(),
                    "Invalid referenced category document type",
                );
                return false;
            }
            true
        };
        return validate_provided_doc(category, "Category", provider, report, category_validator)
            .await;
    }
    Ok(true)
}
