//! Catalyst Signed Documents validation

pub(crate) mod utils;

use catalyst_types::problem_report::ProblemReport;

use crate::{
    doc_types::{CommentDocument, DocumentType, ProposalDocument},
    error::CatalystSignedDocError,
    CatalystSignedDocument, DocumentRef,
};

/// Stateless validation function rule type
pub(crate) type StatelessRule = fn(&CatalystSignedDocument, &ProblemReport) -> bool;
/// Statefull validation function rule type
pub(crate) type StatefullRule<DocType, DocProvider> =
    fn(&DocType, &DocProvider, &ProblemReport) -> bool;

/// Trait for defining a stateless validation rules.
pub trait StatelessValidation
where Self: 'static
{
    /// Stateless validation rules
    const STATELESS_RULES: &[StatelessRule];

    /// Perform a stateless validation, collecting a problem report
    fn validate(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
        Self::STATELESS_RULES
            .iter()
            .map(|rule| rule(doc, report))
            .all(|res| res)
    }
}

/// Trait for defining a statefull validation rules.
pub trait StatefullValidation<DocProvider>
where
    Self: 'static,
    DocProvider: 'static + Fn(&DocumentRef) -> Option<CatalystSignedDocument>,
{
    /// Statefull validation rules
    const STATEFULL_RULES: &[StatefullRule<Self, DocProvider>];

    /// Perform a statefull validation, collecting a problem report
    fn validate(&self, provider: &DocProvider, report: &ProblemReport) -> bool {
        Self::STATEFULL_RULES
            .iter()
            .map(|rule| rule(self, provider, report))
            .all(|res| res)
    }
}

/// A comprehensive validation of the `CatalystSignedDocument`,
/// including a signature verification and document type based validation.
///
/// # Errors
///
/// Returns a report of validation failures and the source error.
pub fn validate<DocProvider>(
    doc: &CatalystSignedDocument, provider: &DocProvider,
) -> Result<(), CatalystSignedDocError>
where DocProvider: 'static + Fn(&DocumentRef) -> Option<CatalystSignedDocument> {
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
                proposal_doc.validate(provider, &report);
            }
        },
        DocumentType::ProposalTemplate => {},
        DocumentType::CommentDocument => {
            if let Ok(comment_doc) = CommentDocument::from_signed_doc(doc, &report) {
                comment_doc.validate(provider, &report);
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
