//! Catalyst Signed Documents validation

use catalyst_types::{id_uri::IdUri, problem_report::ProblemReport};
use rbac_registration::cardano::cip509::SimplePublicKeyType;

use crate::{
    doc_types::{CommentDocument, DocumentType, ProposalDocument},
    error::CatalystSignedDocError,
    CatalystSignedDocument, DocumentRef,
};

/// Trait for validating Catalyst Signed Documents.
pub trait Validator {
    /// Get public keys
    fn get_public_key(&self, kid: &IdUri) -> Option<SimplePublicKeyType>;
    /// Get signed document reference
    fn get_doc_ref(&self, doc_ref: &DocumentRef) -> Option<CatalystSignedDocument>;
}

/// Validation rule
pub struct ValidationRule<T> {
    /// Name of field that is being validated
    #[allow(dead_code)]
    pub(crate) field: String,
    /// Description of what is being validated
    pub(crate) description: String,
    /// Validator function
    pub(crate) validator: fn(&T, &ProblemReport) -> bool,
}

/// A comprehensive validation of the `CatalystSignedDocument`,
/// including a signature verification and document type based validation.
///
/// # Errors
///
/// Returns a report of validation failures and the source error.
pub fn validate<F>(
    doc: &CatalystSignedDocument, doc_getter: impl Validator,
) -> Result<(), CatalystSignedDocError> {
    let error_report = ProblemReport::new("Catalyst Signed Document Validation");

    let doc_type: DocumentType = match doc.doc_type().try_into() {
        Ok(doc_type) => doc_type,
        Err(e) => {
            error_report.invalid_value(
                "`type`",
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
        DocumentType::ProposalDocument => {
            if let Ok(proposal_doc) = ProposalDocument::from_signed_doc(doc, &error_report) {
                proposal_doc.validate_with_report(doc_getter, &error_report);
            }
        },
        DocumentType::ProposalTemplate => {},
        DocumentType::CommentDocument => {
            if let Ok(comment_doc) = CommentDocument::from_signed_doc(doc, &error_report) {
                comment_doc.validate_with_report(&doc_getter, &error_report);
            }
        },
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
