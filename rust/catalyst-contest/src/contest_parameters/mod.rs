//! //! `Contest Parameters` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_parameters/

pub mod rule;

#[cfg(test)]
mod tests;

use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, doc_types::CONTEST_PARAMETERS,
    problem_report::ProblemReport, providers::CatalystSignedDocumentProvider, uuid::UuidV7,
};
use chrono::{DateTime, Utc};

/// `Contest Parameters` document type.
#[derive(Debug, Clone)]
pub struct ContestParameters {
    /// Document reference info
    doc_ref: DocumentRef,
    /// Contest Parameters payload
    payload: ContestParametersPayload,
    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    report: ProblemReport,
}

impl PartialEq for ContestParameters {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.doc_ref.eq(&other.doc_ref)
    }
}

/// Content Parameters JSON payload type.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(crate) struct ContestParametersPayload {
    /// Contest start date
    pub(crate) start: DateTime<Utc>,
    /// Contest end date
    pub(crate) end: DateTime<Utc>,
}

impl ContestParameters {
    /// Returns 'id' metadata field
    #[must_use]
    pub fn id(&self) -> &UuidV7 {
        self.doc_ref.id()
    }

    /// Returns 'ver' metadata field
    #[must_use]
    pub fn ver(&self) -> &UuidV7 {
        self.doc_ref.ver()
    }

    /// Returns contest start date
    #[must_use]
    pub fn start(&self) -> &DateTime<Utc> {
        &self.payload.start
    }

    /// Returns contest end date
    #[must_use]
    pub fn end(&self) -> &DateTime<Utc> {
        &self.payload.end
    }

    /// Returns `ProblemReport`
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }
}

impl ContestParameters {
    /// Trying to build Contest Parameters document collecting all issues into the
    /// `report`.
    ///
    /// # Errors
    ///  - If provided document not a Contest Parameters type
    ///  - If provided document is invalid (`report().is_problematic()`)
    ///  - `provider` returns error
    pub fn new<Provider>(
        doc: &CatalystSignedDocument,
        _provider: &Provider,
    ) -> anyhow::Result<Self>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        if doc.report().is_problematic() {
            anyhow::bail!("Provided document is not valid {:?}", doc.report())
        }
        anyhow::ensure!(
            doc.doc_type()? == &CONTEST_PARAMETERS,
            "Document must be Contest Parameters type"
        );

        let report = ProblemReport::new("Contest Parameters");

        let (payload, _) = get_payload(doc, &report);

        Ok(ContestParameters {
            doc_ref: doc.doc_ref()?,
            payload,
            report,
        })
    }
}

/// Get `ContestParametersPayload` from the provided `CatalystSignedDocument`, fill the
/// provided `ProblemReport` if something goes wrong.
/// Returns additional boolean flag, was it valid or not.
pub(crate) fn get_payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> (ContestParametersPayload, bool) {
    let mut valid = true;
    let payload = doc
            .decoded_content()
            .inspect_err(|_| {
                report.functional_validation(
                    "Invalid Document content, cannot get decoded bytes",
                    "Cannot get a document content during Contest Parameters document validation.",
                );
                valid = false;
            })
            .and_then(|v | Ok(serde_json::from_slice::<ContestParametersPayload>(&v)?))
            .inspect_err(|_| {
                report.functional_validation(
                    "Invalid Document content, must be a valid JSON object compliant with the JSON schema.",
                    "Cannot get a document content during Contest Parameters document validation.",
                );
                valid = false;
            })
            .unwrap_or_default();

    if payload.start >= payload.end {
        report.functional_validation(
            "Invalid Document content, end date must be after the start date.",
            "Cannot get a document content during Contest Parameters document validation.",
        );
        valid = false;
    }

    (payload, valid)
}
