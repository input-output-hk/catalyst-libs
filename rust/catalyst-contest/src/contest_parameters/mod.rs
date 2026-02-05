//! `Contest Parameters` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_parameters/

mod payload;
pub mod rule;

#[cfg(test)]
mod tests;

use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, DocumentRefs,
    doc_types::{CONTEST_BALLOT, CONTEST_DELEGATION, CONTEST_PARAMETERS, PROPOSAL},
    problem_report::ProblemReport,
    providers::{
        CatalystSignedDocumentProvider, CatalystSignedDocumentSearchQuery, DocTypeSelector,
        DocumentRefSelector,
    },
    uuid::UuidV7,
};
use chrono::{DateTime, Utc};

use self::payload::ContestParametersPayload;
pub use self::payload::VotingOptions;
use crate::vote_protocol::committee::ElectionPublicKey;

/// `Contest Parameters` document type.
#[derive(Debug, Clone)]
pub struct ContestParameters {
    /// Document reference info
    doc_ref: DocumentRef,
    /// Contest Parameters payload
    payload: ContestParametersPayload,
    /// 'parameters' metadata field
    parameters: DocumentRefs,
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
    /// Returns document reference
    #[must_use]
    pub fn doc_ref(&self) -> &DocumentRef {
        &self.doc_ref
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

    /// Returns contest snapshot taking date
    #[must_use]
    pub fn snapshot(&self) -> &DateTime<Utc> {
        &self.payload.snapshot
    }

    /// Returns contest voting options
    #[must_use]
    pub fn options(&self) -> &VotingOptions {
        &self.payload.options
    }

    /// Returns an election public key.
    #[must_use]
    pub fn election_public_key(&self) -> &ElectionPublicKey {
        &self.payload.election_public_key
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
    pub fn new(
        doc: &CatalystSignedDocument,
        _provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<Self> {
        if doc.report().is_problematic() {
            anyhow::bail!("Provided document is not valid {:?}", doc.report())
        }
        anyhow::ensure!(
            doc.doc_type()? == &CONTEST_PARAMETERS,
            "Document must be Contest Parameters type"
        );

        let report = ProblemReport::new("Contest Parameters");
        let payload = get_payload(doc, &report);
        let parameters = doc
            .doc_meta()
            .parameters()
            .ok_or_else(|| {
                anyhow::anyhow!("'Contest Parameter' document must have 'parameters' field")
            })
            .cloned()?;

        Ok(ContestParameters {
            doc_ref: doc.doc_ref()?,
            payload,
            parameters,
            report,
        })
    }

    /// Timeline verification, based on the 'Contest Parameters' 'start' and 'end' fields.
    /// Filling to provided problem report.
    pub(crate) fn timeline_check(
        &self,
        ver: UuidV7,
        report: &ProblemReport,
        document_name: &str,
    ) {
        if ver.time() > self.end() || ver.time() < self.start() {
            report.functional_validation(
                &format!(
                    "'ver' metadata field must be in 'Contest Parameters' timeline range. 'ver': {}, start: {}, end: {}",
                    ver.time(),
                    self.start(),
                    self.end()
                ),
                &format!("'{document_name}' timeline check"),
            );
        }
    }

    /// Return a list of associated 'Proposal' documents
    /// with the 'Contest Parameters' document.
    ///
    /// # Errors
    ///  - `provider` returns error.
    pub(crate) fn get_associated_proposals(
        &self,
        provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<Vec<CatalystSignedDocument>> {
        let query = CatalystSignedDocumentSearchQuery {
            doc_type: Some(DocTypeSelector::In(vec![PROPOSAL])),
            parameters: Some(DocumentRefSelector::Eq(self.parameters.clone())),
            ..Default::default()
        };
        let proposals = provider.try_search_latest_docs(&query)?;
        //TODO: Leave only proposals with the corresponding 'Proposal Submission Action' final
        // document.

        Ok(proposals)
    }

    /// Return a list of associated 'Contest Ballot' documents
    /// with the 'Contest Parameters' document.
    ///
    /// # Errors
    ///  - `provider` returns error.
    pub(crate) fn get_associated_ballots(
        &self,
        provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<Vec<CatalystSignedDocument>> {
        let query = CatalystSignedDocumentSearchQuery {
            doc_type: Some(DocTypeSelector::In(vec![CONTEST_BALLOT])),
            parameters: Some(DocumentRefSelector::Eq(vec![self.doc_ref.clone()].into())),
            ..Default::default()
        };
        // Consider ONLY latest versions.
        let ballots = provider.try_search_latest_docs(&query)?;
        Ok(ballots)
    }

    /// Return a list of associated 'Contest Delegation' documents
    /// with the 'Contest Parameters' document.
    ///
    /// # Errors
    ///  - `provider` returns error.
    #[allow(dead_code)]
    pub(crate) fn get_associated_delegations(
        &self,
        provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<Vec<CatalystSignedDocument>> {
        let query = CatalystSignedDocumentSearchQuery {
            doc_type: Some(DocTypeSelector::In(vec![CONTEST_DELEGATION])),
            parameters: Some(DocumentRefSelector::Eq(vec![self.doc_ref.clone()].into())),
            ..Default::default()
        };
        let delegations = provider.try_search_latest_docs(&query)?;

        Ok(delegations)
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
