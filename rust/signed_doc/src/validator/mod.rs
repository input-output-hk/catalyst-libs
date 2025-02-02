//! Catalyst Signed Documents validation

use catalyst_types::problem_report::ProblemReport;

use crate::{doc_types::DocumentType, error::CatalystSignedDocError, CatalystSignedDocument};

/// `CatalystSignedDocument` type based specific validation
///
/// # Errors
///
/// Returns a report of validation failures and the source error.
pub fn validate(doc: &CatalystSignedDocument) -> Result<(), CatalystSignedDocError> {
    let error_report = ProblemReport::new("Catalyst Signed Document Validation");

    let doc_type: DocumentType = match doc.doc_type().try_into() {
        Ok(doc_type) => doc_type,
        Err(e) => {
            error_report.invalid_value(
                "document `type`",
                &doc.doc_type().to_string(),
                &e.to_string(),
                "verifying document type",
            );
            return Err(CatalystSignedDocError::new(
                error_report,
                anyhow::anyhow!("Validation of the Catalyst Signed Document failed"),
            ));
        },
    };

    #[allow(clippy::match_same_arms)]
    match doc_type {
        DocumentType::ProposalDocument => {},
        DocumentType::ProposalTemplate => {},
        DocumentType::CommentDocument => {},
        DocumentType::CommentTemplate => {},
        DocumentType::ReviewDocument => {},
        DocumentType::ReviewTemplate => {},
        DocumentType::CategoryParametersDocument => {},
        DocumentType::CategoryParametersTemplate => {},
        DocumentType::CampaignParametersDocument => {},
        DocumentType::CampaignParametersTemplate => {},
        DocumentType::BrandParametersDocument => {},
        DocumentType::BrandParametersTemplate => {},
        DocumentType::ProposalActionDocument => {},
        DocumentType::PublicVoteTxV2 => {},
        DocumentType::PrivateVoteTxV2 => {},
        DocumentType::ImmutableLedgerBlock => {},
    }

    if error_report.is_problematic() {
        return Err(CatalystSignedDocError::new(
            error_report,
            anyhow::anyhow!("Validation of the Catalyst Signed Document failed"),
        ));
    }

    Ok(())
}
