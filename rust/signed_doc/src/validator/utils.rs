//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use crate::{CatalystSignedDocument, DocumentRef, providers::CatalystSignedDocumentProvider};

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(crate) async fn validate_provided_doc<Provider, Validator>(
    doc_ref: &DocumentRef, provider: &Provider, report: &ProblemReport, validator: Validator,
) -> anyhow::Result<bool>
where
    Provider: 'static + CatalystSignedDocumentProvider,
    Validator: Fn(CatalystSignedDocument) -> anyhow::Result<bool>,
{
    if let Some(doc) = provider.try_get_doc(doc_ref).await? {
        validator(doc)
    } else {
        report.functional_validation(
            format!("Cannot retrieve a document {doc_ref}").as_str(),
            "Validation data provider could not return a corresponding {doc_name}.",
        );
        Ok(false)
    }
}
