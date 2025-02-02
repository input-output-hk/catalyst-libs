//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::problem_report::ProblemReport;

use crate::{error::CatalystSignedDocError, CatalystSignedDocument};

/// Proposal document `UuidV4` type.
const PROPOSAL_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);

/// Proposal Document struct
pub struct ProposalDocument {
    /// Proposal document content data
    /// TODO: change it to `serde_json::Value` type
    #[allow(dead_code)]
    content: Vec<u8>,
}

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

        if doc.doc_type().uuid() != PROPOSAL_DOCUMENT_UUID_TYPE {
            error_report.invalid_value(
                "`type`",
                &doc.doc_type().to_string(),
                &format!("Proposal Document type UUID value is {PROPOSAL_DOCUMENT_UUID_TYPE}"),
                CONTEXT,
            );
            failed = true;
        }

        // TODO add other validation

        if failed {
            anyhow::bail!("Failed to build `ProposalDocument` from `CatalystSignedDoc`");
        }

        let content = doc.doc_content().decoded_bytes().to_vec();
        Ok(Self { content })
    }

    /// A comprehensive validation of the `ProposalDocument` content.
    #[allow(clippy::unused_self)]
    pub(crate) fn validate_with_report<F>(&self, _doc_getter: F, _error_report: &ProblemReport)
    where F: FnMut() -> Option<CatalystSignedDocument> {
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
