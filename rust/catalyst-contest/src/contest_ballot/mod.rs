//! `Contest Ballot` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

mod payload;
pub mod rule;
#[cfg(test)]
mod tests;

use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types::CONTEST_BALLOT, problem_report::ProblemReport,
    providers::CatalystSignedDocumentProvider,
};
use catalyst_voting::{
    crypto::hash::{Blake2b512Hasher, digest::Digest},
    vote_protocol::voter::proof::{VoterProofCommitment, verify_voter_proof},
};
use minicbor::{Decode, Encode};

use crate::{
    contest_ballot::payload::{Choices, ContestBallotPayload},
    contest_parameters::{self, ContestParameters},
};

/// An individual Ballot cast in a Contest by a registered user.
pub struct ContestBallot {
    /// A contest ballot payload.
    #[allow(dead_code)]
    payload: Option<ContestBallotPayload>,
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
            anyhow::bail!("Provided document is not valid {:?}", doc.report())
        }
        anyhow::ensure!(
            doc.doc_type()? == &CONTEST_BALLOT,
            "Document must be Contest Ballot type"
        );

        let report = ProblemReport::new("Contest Ballot");

        let payload = payload(doc, &report);
        let params = check_parameters(doc, provider, &report)?;
        if let (Some(payload), Some(params)) = (&payload, &params) {
            check_proof(payload, params, &report)?;
        }

        Ok(Self { payload, report })
    }

    /// Returns a problem report.
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }
}

/// Returns a decoded contest ballot payload.
fn payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> Option<ContestBallotPayload> {
    let Ok(bytes) = doc.decoded_content() else {
        report.functional_validation(
            "Invalid document content, cannot get decoded bytes",
            "Cannot get a document content during Contest Ballot document validation.",
        );
        return None;
    };

    let mut decoder = minicbor::Decoder::new(&bytes);

    let Ok(payload) = ContestBallotPayload::decode(&mut decoder, &mut report.clone()) else {
        report.functional_validation(
            "Invalid document content: unable to decode CBOR",
            "Cannot get a document content during Contest Ballot document validation.",
        );
        return None;
    };

    Some(payload)
}

/// Checks the parameters of a document and returns a contest parameters document.
fn check_parameters(
    doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<Option<CatalystSignedDocument>> {
    let Some(doc_ref) = doc.doc_meta().parameters().and_then(|v| v.first()) else {
        report.missing_field(
            "parameters",
            "Contest Ballot must have a 'parameters' metadata field",
        );
        return Ok(None);
    };

    let Some(contest_parameters) = provider.try_get_doc(doc_ref)? else {
        report.functional_validation(
            &format!("Cannot get referenced document by reference: {doc_ref}"),
            "Missing 'Contest Parameters' document for the Contest Ballot document",
        );
        return Ok(None);
    };

    let Ok(doc_ver) = doc.doc_ver() else {
        report.missing_field(
            "ver",
            "Missing 'ver' metadata field for 'Contest Ballot' document",
        );
        return Ok(Some(contest_parameters));
    };

    ContestParameters::timeline_check(doc_ver, &contest_parameters, report, "Contest Ballot");

    Ok(Some(contest_parameters))
}

/// Checks the proof.
fn check_proof(
    payload: &ContestBallotPayload,
    contest_parameters: &CatalystSignedDocument,
    report: &ProblemReport,
) -> anyhow::Result<()> {
    let election_public_key =
        contest_parameters::get_payload(contest_parameters, report).election_public_key;
    let commitment_key = commitment_key(contest_parameters)?;

    for (index, choice) in &payload.choices {
        let Choices::Encrypted {
            choices,
            row_proof: Some(proof),
        } = choice
        else {
            continue;
        };

        if !verify_voter_proof(
            choices.clone(),
            &election_public_key,
            &commitment_key,
            proof,
        ) {
            report.functional_validation(
                &format!("Failed to verify proof ({index} index)"),
                "'Contest Ballot' document validation",
            );
        }
    }

    Ok(())
}

/// Returns a commitment key calculated from the document reference of the given contest
/// parameters document.
fn commitment_key(
    contest_parameters: &CatalystSignedDocument
) -> anyhow::Result<VoterProofCommitment> {
    let params_ref = contest_parameters.doc_ref()?;
    let mut buffer = Vec::new();
    params_ref.encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())?;
    let mut hasher = Blake2b512Hasher::new();
    hasher.update(&buffer);
    Ok(VoterProofCommitment::from_hash(hasher))
}
