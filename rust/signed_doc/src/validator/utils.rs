//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use super::ValidationDataProvider;
use crate::{CatalystSignedDocument, DocumentRef};

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(crate) fn validate_provided_doc(
    doc_ref: &DocumentRef, doc_name: &str, provider: &dyn ValidationDataProvider,
    report: &ProblemReport, validator: impl Fn(CatalystSignedDocument) -> bool,
) -> bool {
    if let Some(doc) = provider.get_doc_ref(doc_ref) {
        validator(doc)
    } else {
        report.functional_validation(
            format!("Cannot retrieve a {doc_name} document {doc_ref}").as_str(),
            "Validation data provider could not return a corresponding {doc_name}.",
        );
        false
    }
}
