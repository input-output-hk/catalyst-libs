//! Validation utility functions

use catalyst_types::problem_report::ProblemReport;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocumentRef};

/// A helper validation document function, which validates a document from the
/// `ValidationDataProvider`.
pub(crate) async fn validate_provided_doc<Provider, Validator>(
    doc_ref: &DocumentRef, provider: &Provider, report: &ProblemReport, validator: Validator,
) -> anyhow::Result<bool>
where
    Provider: CatalystSignedDocumentProvider,
    Validator: Fn(CatalystSignedDocument) -> bool,
{
    const CONTEXT: &str = "Validation data provider";

    // General check for document ref

    // Getting the Signed Document instance from a doc ref.
    // The reference document must exist
    if let Some(doc) = provider.try_get_doc(doc_ref).await? {
        let id = doc
            .doc_id()
            .inspect_err(|_| report.missing_field("id", CONTEXT))?;

        let ver = doc
            .doc_ver()
            .inspect_err(|_| report.missing_field("ver", CONTEXT))?;
        // id and version must match the values in ref doc
        if &id != doc_ref.id() && &ver != doc_ref.ver() {
            report.invalid_value(
                "id and version",
                &format!("id: {id}, ver: {ver}"),
                &format!("id: {}, ver: {}", doc_ref.id(), doc_ref.ver()),
                CONTEXT,
            );
            return Ok(false);
        }
        Ok(validator(doc))
    } else {
        report.functional_validation(
            format!("Cannot retrieve a document {doc_ref}").as_str(),
            CONTEXT,
        );
        Ok(false)
    }
}
