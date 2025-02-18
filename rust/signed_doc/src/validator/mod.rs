//! Catalyst Signed Documents validation

pub(crate) mod rules;
pub(crate) mod utils;

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
    ) -> BoxFuture<'a, anyhow::Result<bool>>;
}

/// A comprehensive validation of the `CatalystSignedDocument`,
/// including a signature verification and document type based validation.
/// Return true if all signatures are valid, otherwise return false.
/// Also it imediatly return false, if document is already invalid.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument, provider: &Provider,
) -> anyhow::Result<bool>
where Provider: 'static + CatalystSignedDocumentProvider {
    if doc.report().is_problematic() {
        return Ok(false);
    }

    let rules = validation_rules(doc)?;

    validate_rules(rules, doc, provider).await
}

/// Running a validation by the provided list of rules.
///
/// # Errors
///
/// If `provider` returns error, fails fast throwing that error.
async fn validate_rules<Provider>(
    rules: Vec<Box<dyn ValidationRule<Provider>>>, doc: &CatalystSignedDocument,
    provider: &Provider,
) -> anyhow::Result<bool>
where
    Provider: 'static + CatalystSignedDocumentProvider,
{
    let checks = rules.iter().map(|rule| rule.check(doc, provider));
    for res in futures::future::join_all(checks).await {
        if !(res?) {
            return Ok(false);
        }
    }
    Ok(true)
}
