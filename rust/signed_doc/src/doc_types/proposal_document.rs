//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};

use crate::{
    doc_types::PROPOSAL_TEMPLATE_UUID_TYPE,
    error::CatalystSignedDocError,
    validator::{ValidationRule, Validator},
    CatalystSignedDocument,
};

/// Proposal document `UuidV4` type.
pub const PROPOSAL_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);

/// Proposal Document struct
pub struct ProposalDocument;

impl ProposalDocument {
    /// Try to build `ProposalDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    #[allow(dead_code)]
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, error_report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        /// Context for error messages.
        const CONTEXT: &str = "Catalyst Signed Document to Proposal Document";
        let mut failed = false;

        let rules = vec![
            ValidationRule {
                field: "type".to_string(),
                description: format!(
                    "Proposal Document type UUID value is {PROPOSAL_DOCUMENT_UUID_TYPE}"
                ),
                validator: |doc: &CatalystSignedDocument, _| {
                    doc.doc_type().uuid() != PROPOSAL_DOCUMENT_UUID_TYPE
                },
            },
            ValidationRule {
                field: "template".to_string(),
                description: format!(
                    "Proposal Document template UUID value is {PROPOSAL_TEMPLATE_UUID_TYPE}"
                ),
                validator: |doc: &CatalystSignedDocument, _| {
                    doc.doc_type().uuid() != PROPOSAL_TEMPLATE_UUID_TYPE
                },
            },
        ];
        for rule in rules {
            if !(rule.validator)(doc, error_report) {
                error_report.functional_validation(&rule.description, "");
                failed = true;
            }
        }

        // TODO add other validation

        if failed {
            anyhow::bail!("Failed to build `ProposalDocument` from `CatalystSignedDoc`");
        }

        Ok(Self)
    }

    /// A comprehensive validation of the `ProposalDocument` content.
    #[allow(clippy::unused_self)]
    pub(crate) fn validate_with_report(
        &self, _validator: impl Validator, _error_report: &ProblemReport,
    ) {
        // TODO: implement the rest of the validation
    }
}

impl TryFrom<CatalystSignedDocument> for ProposalDocument {
    type Error = CatalystSignedDocError;

    fn try_from(doc: CatalystSignedDocument) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Proposal Document");
        let res = Self::from_signed_doc(&doc, &error_report)
            .map_err(|e| CatalystSignedDocError::new(error_report, e))?;
        Ok(res)
    }
}
