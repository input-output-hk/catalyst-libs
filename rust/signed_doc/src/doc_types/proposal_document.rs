//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};

use super::{CATEGORY_DOCUMENT_UUID_TYPE, PROPOSAL_TEMPLATE_UUID_TYPE};
use crate::{
    error::CatalystSignedDocError,
    metadata::{ContentEncoding, ContentType},
    validator::{
        utils::validate_provided_doc, StatefullRule, StatelessRule, ValidationDataProvider,
        Validator,
    },
    CatalystSignedDocument, DocumentRef,
};

/// Proposal document `UuidV4` type.
pub const PROPOSAL_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);

/// Proposal Document struct
pub struct ProposalDocument {
    /// `template` doc ref
    template: DocumentRef,
    /// `category` doc ref
    category: Option<DocumentRef>,
}

impl Validator for ProposalDocument {
    const STATEFULL_RULES: &[StatefullRule<Self>] = &[template_full_check, category_full_check];
    const STATELESS_RULES: &[StatelessRule] = &[
        type_check,
        content_type_check,
        content_encoding_check,
        template_check,
    ];
}

/// `type` field validation
fn type_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
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
fn content_type_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
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
fn content_encoding_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
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
fn template_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_meta().template().is_none() {
        report.missing_field("template", "Proposal Document must have a template field");
        return false;
    }
    true
}

/// `template` statefull validation
fn template_full_check(
    doc: &ProposalDocument, provider: &dyn ValidationDataProvider, report: &ProblemReport,
) -> bool {
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
        true
    };
    validate_provided_doc(
        &doc.template,
        "Proposal Template",
        provider,
        report,
        template_validator,
    )
}

/// `category_id` statefull validation
fn category_full_check(
    doc: &ProposalDocument, provider: &dyn ValidationDataProvider, report: &ProblemReport,
) -> bool {
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
        return validate_provided_doc(category, "Category", provider, report, category_validator);
    }
    true
}

impl ProposalDocument {
    /// Try to build `ProposalDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        if Self::stateless_validation(doc, report) {
            anyhow::bail!("Failed to build `ProposalDocument` from `CatalystSignedDoc`");
        }

        let category = doc.doc_meta().category_id();
        let template = doc
            .doc_meta()
            .template()
            .ok_or(anyhow::anyhow!("missing `template` field"))?;

        Ok(Self { template, category })
    }
}

impl TryFrom<CatalystSignedDocument> for ProposalDocument {
    type Error = CatalystSignedDocError;

    fn try_from(doc: CatalystSignedDocument) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Catalyst Signed Document to Proposal Document");
        let res = Self::from_signed_doc(&doc, &error_report)
            .map_err(|e| CatalystSignedDocError::new(error_report, e))?;
        Ok(res)
    }
}
