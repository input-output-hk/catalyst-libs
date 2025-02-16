//! Catalyst Signed Documents validation

pub(crate) mod utils;

use catalyst_types::{id_uri::IdUri, problem_report::ProblemReport};
use rbac_registration::cardano::cip509::SimplePublicKeyType;

use crate::{
    doc_types::{CommentDocument, DocumentType, ProposalDocument},
    error::CatalystSignedDocError,
    CatalystSignedDocument, DocumentRef,
};

/// Trait for getting a necessary data needed during the validation process.
pub trait ValidationDataProvider {
    /// Get public keys
    fn get_public_key(&self, kid: &IdUri) -> Option<SimplePublicKeyType>;
    /// Get signed document by document reference
    fn get_doc_ref(&self, doc_ref: &DocumentRef) -> Option<CatalystSignedDocument>;
}

/// Stateless validation function rule type
pub(crate) type StatelessRule = fn(&CatalystSignedDocument, &ProblemReport) -> bool;
/// Statefull validation function rule type
pub(crate) type StatefullRule<T> = fn(&T, &dyn ValidationDataProvider, &ProblemReport) -> bool;

/// Trait for defining a validation rules.
pub trait Validator
where Self: 'static
{
    /// Stateless validation rules
    const STATELESS_RULES: &[StatelessRule];
    /// Statefull validation rules
    const STATEFULL_RULES: &[StatefullRule<Self>];

    /// Perform a stateless validation, collecting a problem report
    fn stateless_validation(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
        Self::STATELESS_RULES
            .iter()
            .map(|rule| rule(doc, report))
            .all(|res| res)
    }

    /// Perform a statefull validation, collecting a problem report
    fn statefull_validation(
        &self, provider: &impl ValidationDataProvider, report: &ProblemReport,
    ) -> bool {
        Self::STATEFULL_RULES
            .iter()
            .map(|rule| rule(self, provider, report))
            .all(|res| res)
    }
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
    doc: &CatalystSignedDocument, doc_getter: &impl ValidationDataProvider,
) -> Result<(), CatalystSignedDocError> {
    let report = ProblemReport::new("Catalyst Signed Document Validation");

    let doc_type: DocumentType = match doc.doc_type().try_into() {
        Ok(doc_type) => doc_type,
        Err(e) => {
            report.invalid_value(
                "`type`",
                &doc.doc_type().to_string(),
                &e.to_string(),
                "verifying document type",
            );
            return Err(CatalystSignedDocError::new(
                report,
                anyhow::anyhow!("Validation of the Catalyst Signed Document failed"),
            ));
        },
    };

    #[allow(clippy::match_same_arms)]
    match doc_type {
        DocumentType::ProposalDocument => {
            if let Ok(proposal_doc) = ProposalDocument::from_signed_doc(doc, &report) {
                proposal_doc.statefull_validation(doc_getter, &report);
            }
        },
        DocumentType::ProposalTemplate => {},
        DocumentType::CommentDocument => {
            if let Ok(comment_doc) = CommentDocument::from_signed_doc(doc, &report) {
                comment_doc.validate_with_report(doc_getter, &report);
            }
        },
        DocumentType::CommentTemplate => {},
        DocumentType::ReviewDocument => {},
        DocumentType::ReviewTemplate => {},
        DocumentType::CategoryDocument => {},
        DocumentType::CategoryTemplate => {},
        DocumentType::CampaignParametersDocument => {},
        DocumentType::CampaignParametersTemplate => {},
        DocumentType::BrandParametersDocument => {},
        DocumentType::BrandParametersTemplate => {},
        DocumentType::ProposalActionDocument => {},
        DocumentType::PublicVoteTxV2 => {},
        DocumentType::PrivateVoteTxV2 => {},
        DocumentType::ImmutableLedgerBlock => {},
    }

    if report.is_problematic() {
        return Err(CatalystSignedDocError::new(
            report,
            anyhow::anyhow!("Validation of the Catalyst Signed Document failed"),
        ));
    }

    Ok(())
}
