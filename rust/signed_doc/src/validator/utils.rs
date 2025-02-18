//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use super::ValidationRule;
use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocumentRef};

/// Wrap a provider `rule` into the `Box<dyn ValidationRule>`
pub(super) fn boxed_rule<T, Provider>(rule: T) -> Box<dyn ValidationRule<Provider>>
where
    Provider: 'static + CatalystSignedDocumentProvider,
    T: 'static + ValidationRule<Provider>,
{
    Box::new(rule)
}

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(super) async fn validate_provided_doc<Provider, Validator>(
    doc_ref: &DocumentRef, provider: &Provider, report: &ProblemReport, validator: Validator,
) -> anyhow::Result<bool>
where
    Provider: 'static + CatalystSignedDocumentProvider,
    Validator: Fn(CatalystSignedDocument) -> bool,
{
    if let Some(doc) = provider.try_get_doc(doc_ref).await? {
        Ok(validator(doc))
    } else {
        report.functional_validation(
            format!("Cannot retrieve a document {doc_ref}").as_str(),
            "Validation data provider could not return a corresponding {doc_name}.",
        );
        Ok(false)
    }
}
