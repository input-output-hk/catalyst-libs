//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};

use super::{CATEGORY_DOCUMENT_UUID_TYPE, PROPOSAL_TEMPLATE_UUID_TYPE};
use crate::{
    error::CatalystSignedDocError,
    metadata::{ContentEncoding, ContentType},
    validator::{utils::validate_provided_doc, ValidationDataProvider},
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

impl ProposalDocument {
    /// Try to build `ProposalDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        let mut failed = false;

        if doc.doc_type().uuid() != PROPOSAL_DOCUMENT_UUID_TYPE {
            report.invalid_value(
                "type",
                doc.doc_type().to_string().as_str(),
                PROPOSAL_DOCUMENT_UUID_TYPE.to_string().as_str(),
                "Invalid Proposal Document type UUID value",
            );
            failed = true;
        }

        if doc.doc_content_type() != ContentType::Json {
            report.invalid_value(
                "content-type",
                doc.doc_content_type().to_string().as_str(),
                ContentType::Json.to_string().as_str(),
                "Invalid Proposal Document content-type value",
            );
            failed = true;
        }

        if let Some(content_encoding) = doc.doc_content_encoding() {
            if content_encoding != ContentEncoding::Brotli {
                report.invalid_value(
                    "content-encoding",
                    content_encoding.to_string().as_str(),
                    ContentEncoding::Brotli.to_string().as_str(),
                    "Invalid Proposal Document content-encoding value",
                );
                failed = true;
            }
        } else {
            report.missing_field(
                "content-encoding",
                "Proposal Document must have a content-encoding field",
            );
            failed = true;
        }

        let category = doc.doc_meta().category_id();

        let Some(template) = doc.doc_meta().template() else {
            report.missing_field(
                "template",
                "Proposal Document must have a template
        field",
            );
            anyhow::bail!("Failed to build `ProposalDocument` from `CatalystSignedDoc`");
        };

        if failed {
            anyhow::bail!("Failed to build `ProposalDocument` from `CatalystSignedDoc`");
        }

        Ok(Self { template, category })
    }

    /// A comprehensive statefull validation of the `ProposalDocument` content.
    pub(crate) fn validate_with_report(
        &self, provider: &impl ValidationDataProvider, report: &ProblemReport,
    ) {
        let template_validator = |template_doc: CatalystSignedDocument| {
            if template_doc.doc_type().uuid() != PROPOSAL_TEMPLATE_UUID_TYPE {
                report.invalid_value(
                    "template",
                    template_doc.doc_type().to_string().as_str(),
                    PROPOSAL_TEMPLATE_UUID_TYPE.to_string().as_str(),
                    "Invalid referenced template document type",
                );
            }
        };
        validate_provided_doc(
            &self.template,
            "Proposal Template",
            provider,
            report,
            template_validator,
        );

        if let Some(category) = &self.category {
            let category_validator = |category_doc: CatalystSignedDocument| {
                if category_doc.doc_type().uuid() != CATEGORY_DOCUMENT_UUID_TYPE {
                    report.invalid_value(
                        "category_id",
                        category_doc.doc_type().to_string().as_str(),
                        CATEGORY_DOCUMENT_UUID_TYPE.to_string().as_str(),
                        "Invalid referenced category document type",
                    );
                }
            };
            validate_provided_doc(category, "Category", provider, report, category_validator);
        }
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
