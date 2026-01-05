//! An individual Ballot cast in a Contest by a registered user.

use anyhow::{bail, ensure};
use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types::CONTEST_BALLOT, problem_report::ProblemReport,
    providers::CatalystSignedDocumentProvider,
};
use minicbor::Decode;

use crate::ContentBallotPayload;

/// An individual Ballot cast in a Contest by a registered user.
pub struct ContestBallot {
    /// A contest ballot payload.
    payload: Option<ContentBallotPayload>,
    /// A report containing all the issues occurred during `ContestBallot` validation.
    report: ProblemReport,
}

impl ContestBallot {
    /// Creates a new `ContestBallot` instance.
    ///
    /// # Errors
    /// - Wrong document type (`CONTEST_BALLOT` is expected).
    /// - Invalid document (`report().is_problematic() == true`).
    /// - `Provider` error.
    pub fn new<Provider>(
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<Self>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        if doc.report().is_problematic() {
            bail!("Provided document is not valid {:?}", doc.report())
        }
        ensure!(
            doc.doc_type()? == &CONTEST_BALLOT,
            "Document must be Contest Ballot type"
        );

        let report = ProblemReport::new("Contest Ballot");

        // TODO: Validate parameters.
        let payload = payload(doc, &report);

        Ok(Self { payload, report })
    }

    /// Returns a contest ballot payload.
    #[must_use]
    pub fn payload(&self) -> Option<&ContentBallotPayload> {
        self.payload.as_ref()
    }

    /// Returns a problem report.
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }
}

/// Returns a decoded contest ballot payload.
pub fn payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> Option<ContentBallotPayload> {
    let Ok(bytes) = doc.decoded_content() else {
        report.functional_validation(
            "Invalid document content, cannot get decoded bytes",
            "Cannot get a document content during Contest Ballot document validation.",
        );
        return None;
    };

    let mut decoder = minicbor::Decoder::new(&bytes);
    // TODO: Pass a problem report in the decode context?
    let Ok(payload) = ContentBallotPayload::decode(&mut decoder, &mut ()) else {
        report.functional_validation(
            "Invalid document content: unable to decode CBOR",
            "Cannot get a document content during Contest Ballot document validation.",
        );
        return None;
    };

    Some(payload)
}
