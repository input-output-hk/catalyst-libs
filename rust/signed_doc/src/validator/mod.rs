//! Catalyst Signed Documents validation

pub(crate) mod rules;
pub(crate) mod utils;

use catalyst_types::problem_report::ProblemReport;
use futures::future::BoxFuture;
use rules::{comment_document_rules, proposal_document_rules};
use utils::boxed_rule;

use crate::{
    doc_types::DocumentType, error::CatalystSignedDocError,
    providers::CatalystSignedDocumentProvider, CatalystSignedDocument,
};

/// Trait for defining a single validation rule.
pub(crate) trait ValidationRule<Provider>
where Provider: 'static + CatalystSignedDocumentProvider
{
    /// Perform a validation rule, collecting a problem report
    ///
    /// # Errors
    /// Returns an error if `provider` return an error.
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, provider: &'a Provider,
        report: &'a ProblemReport,
    ) -> BoxFuture<'a, anyhow::Result<bool>>;
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
    let rules = match doc_type {
        DocumentType::ProposalDocument => proposal_document_rules(),
        DocumentType::CommentDocument => comment_document_rules(),
        DocumentType::ProposalTemplate
        | DocumentType::CommentTemplate
        | DocumentType::ReviewDocument
        | DocumentType::ReviewTemplate
        | DocumentType::CategoryDocument
        | DocumentType::CategoryTemplate
        | DocumentType::CampaignParametersDocument
        | DocumentType::CampaignParametersTemplate
        | DocumentType::BrandParametersDocument
        | DocumentType::BrandParametersTemplate
        | DocumentType::ProposalActionDocument
        | DocumentType::PublicVoteTxV2
        | DocumentType::PrivateVoteTxV2
        | DocumentType::ImmutableLedgerBlock => {
            vec![]
        },
    };
    validate_rules(rules, doc, provider, report).await?;
    Ok(())
}

/// Something
async fn validate_rules<Provider>(
    rules: Vec<Box<dyn ValidationRule<Provider>>>, doc: &CatalystSignedDocument,
    provider: &Provider, report: &ProblemReport,
) -> anyhow::Result<bool>
where
    Provider: 'static + CatalystSignedDocumentProvider,
{
    let checks = rules.iter().map(|rule| rule.check(doc, provider, report));
    for res in futures::future::join_all(checks).await {
        if !(res?) {
            return Ok(false);
        }
    }
    Ok(true)
}
