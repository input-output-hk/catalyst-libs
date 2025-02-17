//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocumentRef};

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(crate) async fn validate_provided_doc<Provider, Validator>(
    doc_ref: &DocumentRef, doc_name: &str, provider: &Provider, report: &ProblemReport,
    validator: Validator,
) -> anyhow::Result<bool>
where
    Provider: 'static + CatalystSignedDocumentProvider,
    Validator: Fn(CatalystSignedDocument) -> bool,
{
    if let Some(doc) = provider.try_get_doc(doc_ref).await? {
        Ok(validator(doc))
    } else {
        report.functional_validation(
            format!("Cannot retrieve a {doc_name} document {doc_ref}").as_str(),
            "Validation data provider could not return a corresponding {doc_name}.",
        );
        Ok(false)
    }
}
