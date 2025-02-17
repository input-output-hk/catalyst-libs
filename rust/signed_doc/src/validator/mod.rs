//! Catalyst Signed Documents validation

pub(crate) mod utils;

use std::future::Future;

use catalyst_types::problem_report::ProblemReport;
use futures::future::BoxFuture;

use crate::{
    doc_types::{CommentDocument, DocumentType, ProposalDocument},
    error::CatalystSignedDocError,
    providers::CatalystSignedDocumentProvider,
    CatalystSignedDocument,
};

/// Stateless validation function rule type
pub type StatelessRule = fn(&CatalystSignedDocument, &ProblemReport) -> bool;

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
pub trait StatefullValidation<Provider>
where
    Self: 'static,
    Provider: 'static + CatalystSignedDocumentProvider,
{
    /// Statefull validation rules list
    fn rules<'a>(
        &'a self, provider: &'a Provider, report: &'a ProblemReport,
    ) -> Vec<BoxFuture<'a, anyhow::Result<bool>>>;

    /// Perform a statefull validation, collecting a problem report
    ///
    /// # Errors
    /// Returns an error if `provider` return an error.
    fn validate(
        &self, provider: &Provider, report: &ProblemReport,
    ) -> impl Future<Output = anyhow::Result<bool>> {
        async {
            for res in futures::future::join_all(self.rules(provider, report)).await {
                if !(res?) {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

/// A comprehensive validation of the `CatalystSignedDocument`,
/// including a signature verification and document type based validation.
///
/// # Errors
///
/// Returns a report of validation failures and the source error.
/// If `provider` returns error, fails fast and placed this error into
/// `CatalystSignedDocError::error`.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument, provider: &Provider,
) -> Result<(), CatalystSignedDocError>
where Provider: 'static + CatalystSignedDocumentProvider {
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
                anyhow::anyhow!("Validation of the Catalyst Signed Document failed, {e}"),
            ));
        },
    };

    match validate_inner(doc_type, doc, provider, &report).await {
        Ok(()) if report.is_problematic() => {
            Err(CatalystSignedDocError::new(
                report,
                anyhow::anyhow!("Validation of the Catalyst Signed Document failed"),
            ))
        },
        Err(e) => Err(CatalystSignedDocError::new(report, e)),
        Ok(()) => Ok(()),
    }
}

/// A comprehensive type based validation of the `CatalystSignedDocument`, collecting a
/// `report`.
///
/// # Errors
///
/// If `provider` returns error, fails fast and placed this error into
/// `CatalystSignedDocError::error`.
async fn validate_inner<Provider>(
    doc_type: DocumentType, doc: &CatalystSignedDocument, provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<()>
where
    Provider: 'static + CatalystSignedDocumentProvider,
{
    #[allow(clippy::match_same_arms)]
    match doc_type {
        DocumentType::ProposalDocument => {
            let doc = ProposalDocument::from_signed_doc(doc, report)?;
            doc.validate(provider, report).await?;
        },
        DocumentType::ProposalTemplate => {},
        DocumentType::CommentDocument => {
            let doc = CommentDocument::from_signed_doc(doc, report)?;
            doc.validate(provider, report).await?;
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
    Ok(())
}
