//! Catalyst Signed Documents validation

pub(crate) mod rules;
pub(crate) mod utils;

use catalyst_types::problem_report::ProblemReport;
use futures::future::BoxFuture;

use crate::{
    doc_types::validation_rules, providers::CatalystSignedDocumentProvider, CatalystSignedDocument,
};

/// Trait for defining a single validation rule.
pub(crate) trait ValidationRule<Provider>
where Provider: 'static + CatalystSignedDocumentProvider
{
    /// Perform a  validation rule, collecting a problem report
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
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument, provider: &Provider,
) -> anyhow::Result<bool>
where Provider: 'static + CatalystSignedDocumentProvider {
    let report = ProblemReport::new("Catalyst Signed Document Validation");

    let rules = validation_rules(doc, &report);

    validate_rules(rules, doc, provider, &report).await
}

/// Running a validation by the provided list of rules.
///
/// # Errors
///
/// If `provider` returns error, fails fast throwing that error.
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
