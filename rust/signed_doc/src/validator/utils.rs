//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use crate::{CatalystSignedDocument, DocumentRef};

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(crate) fn validate_provided_doc<DocProvider, Validator>(
    doc_ref: &DocumentRef, doc_name: &str, provider: &DocProvider, report: &ProblemReport,
    validator: Validator,
) -> bool
where
    DocProvider: Fn(&DocumentRef) -> Option<CatalystSignedDocument>,
    Validator: Fn(CatalystSignedDocument) -> bool,
{
    if let Some(doc) = provider(doc_ref) {
        validator(doc)
    } else {
        report.functional_validation(
            format!("Cannot retrieve a {doc_name} document {doc_ref}").as_str(),
            "Validation data provider could not return a corresponding {doc_name}.",
        );
        false
    }
}
