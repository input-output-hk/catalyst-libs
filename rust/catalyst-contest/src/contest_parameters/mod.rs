//! `Contest Parameters` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_parameters/

mod payload;
pub mod rule;

mod serde_group_element;
#[cfg(test)]
mod tests;

use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, doc_types::CONTEST_PARAMETERS,
    problem_report::ProblemReport, providers::CatalystSignedDocumentProvider, uuid::UuidV7,
};
use catalyst_voting::{crypto::group::GroupElement, vote_protocol::committee::ElectionPublicKey};
use chrono::{DateTime, Utc};

use crate::contest_parameters::payload::{Choices, ContestParametersPayload};

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

    /// Returns contest choices
    #[must_use]
    pub fn choices(&self) -> &Choices {
        &self.payload.choices
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
        let payload = get_payload(doc, &report);

        Ok(ContestParameters {
            doc_ref: doc.doc_ref()?,
            payload,
            report,
        })
    }

    /// Timeline verification, based on the 'Contest Parameters' 'start' and 'end' fields.
    /// Filling to provided problem report.
    pub(crate) fn timeline_check(
        ver: UuidV7,
        contest_parameters: &CatalystSignedDocument,
        report: &ProblemReport,
        document_name: &str,
    ) {
        let contest_parameters_payload = get_payload(contest_parameters, report);
        if ver.time() > &contest_parameters_payload.end
            || ver.time() < &contest_parameters_payload.start
        {
            report.functional_validation(
                &format!(
                    "'ver' metadata field must be in 'Contest Parameters' timeline range. 'ver': {}, start: {}, end: {}",
                    ver.time(),
                    contest_parameters_payload.start,
                    contest_parameters_payload.end
                ),
                &format!("'{document_name}' timeline check"),
            );
        }
    }
}

impl Default for ContestParametersPayload {
    fn default() -> Self {
        Self {
            start: DateTime::default(),
            end: DateTime::default(),
            election_public_key: GroupElement::zero().into(),
        }
    }
}

/// Get `ContestParametersPayload` from the provided `CatalystSignedDocument`, fill the
/// provided `ProblemReport` if something goes wrong.
pub(crate) fn get_payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> ContestParametersPayload {
    let payload = doc
        .decoded_content()
        .inspect_err(|_| {
            report.functional_validation(
                "Invalid Document content, cannot get decoded bytes",
                "Cannot get a document content during Contest Parameters document validation.",
            );
        })
        .and_then(|v| Ok(serde_json::from_slice::<ContestParametersPayload>(&v)?))
        .inspect_err(|e| {
            report.functional_validation(
                &e.to_string(),
                "Cannot get a document content during Contest Parameters document validation.",
            );
        })
        .unwrap_or_default();

    if payload.start >= payload.end {
        report.functional_validation(
            "Invalid Document content, end date must be after the start date.",
            "Cannot get a document content during Contest Parameters document validation.",
        );
    }

    payload
}
