//! `Contest Ballot` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

pub mod payload;
pub mod rule;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use anyhow::Context;
use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, catalyst_id::CatalystId, doc_types::CONTEST_BALLOT,
    problem_report::ProblemReport, providers::CatalystSignedDocumentProvider,
};
use catalyst_voting::{
    crypto::hash::{Blake2b512Hasher, digest::Digest},
    vote_protocol::voter::proof::{VoterProofCommitment, verify_voter_proof},
};
use minicbor::{Decode, Encode};

use crate::{
    contest_ballot::payload::{Choices, ContestBallotPayload},
    contest_parameters::ContestParameters,
};

/// An individual Ballot cast in a Contest by a registered user.
pub struct ContestBallot {
    /// Document reference info
    doc_ref: DocumentRef,
    /// A corresponding `CatalystId` of the voter (author of the document).
    voter: CatalystId,
    /// A contest ballot choices per proposal.
    choices: HashMap<DocumentRef, Choices>,
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
    pub fn new(
        doc: &CatalystSignedDocument,
        provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<Self> {
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
        let mut choices = HashMap::new();
        if let Some(params) = &params {
            choices = check_choices(doc, &payload, params, &report)?;
        }

        Ok(Self {
            doc_ref: doc.doc_ref()?,
            voter: doc
                .authors()
                .into_iter()
                .next()
                .context("Contest Ballot document must have only one author/signer")?,
            choices,
            report,
        })
    }

    /// Returns document reference
    #[must_use]
    pub fn doc_ref(&self) -> &DocumentRef {
        &self.doc_ref
    }

    /// Returns 'voter'.
    #[must_use]
    pub fn voter(&self) -> &CatalystId {
        &self.voter
    }

    /// Returns Contest Ballot choices made for proposal
    #[must_use]
    pub fn get_choices_for_proposal(
        &self,
        p_ref: &DocumentRef,
    ) -> Option<&Choices> {
        self.choices.get(p_ref)
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
) -> ContestBallotPayload {
    doc.decoded_content()
        .inspect_err(|_| {
            report.functional_validation(
                "Invalid document content, cannot get decoded bytes",
                "Cannot get a document content during Contest Ballot document validation.",
            );
        })
        .and_then(|bytes| {
            Ok(ContestBallotPayload::decode(
                &mut minicbor::Decoder::new(&bytes),
                &mut report.clone(),
            )?)
        })
        .inspect_err(|_| {
            report.functional_validation(
                "Invalid document content: unable to decode CBOR",
                "Cannot get a document content during Contest Ballot document validation.",
            );
        })
        .unwrap_or_default()
}

/// Checks the document against the 'Contest Parameters' and returns it as a result.
fn check_parameters(
    doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<Option<ContestParameters>> {
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
    let contest_parameter = ContestParameters::new(&contest_parameters, provider)?;

    let Ok(doc_ver) = doc.doc_ver() else {
        report.missing_field(
            "ver",
            "Missing 'ver' metadata field for 'Contest Ballot' document",
        );
        return Ok(Some(contest_parameter));
    };

    contest_parameter.timeline_check(doc_ver, report, "Contest Ballot");

    Ok(Some(contest_parameter))
}

/// Checks choices either they are encrypted or not.
fn check_choices(
    doc: &CatalystSignedDocument,
    payload: &ContestBallotPayload,
    contest_parameters: &ContestParameters,
    report: &ProblemReport,
) -> anyhow::Result<HashMap<DocumentRef, Choices>> {
    let commitment_key = commitment_key(contest_parameters.doc_ref())?;
    if let Some(doc_ref) = doc.doc_meta().doc_ref() {
        let choices = doc_ref
            .iter()
            .zip(payload.choices.values())
            .inspect(|(proposal_ref, choice)| {
                check_choice(
                    proposal_ref,
                    choice,
                    contest_parameters,
                    &commitment_key,
                    report,
                );
            })
            .map(|(p, c)| (p.clone(), c.clone()))
            .collect::<HashMap<_, _>>();
        if choices.len() != doc_ref.len() {
            report.invalid_value(
                "choices",
                &choices.len().to_string(),
                &doc_ref.len().to_string(),
                "The number of referenced 'Proposal' documents in 'ref' field must align with payload's choices",
            );
        }
        Ok(choices)
    } else {
        report.missing_field("ref", "'Contest Ballot' must have a ref metadata field");
        Ok(HashMap::new())
    }
}

/// Verifies an individual choice filling the provided problem report
fn check_choice(
    proposal_ref: &DocumentRef,
    choice: &Choices,
    contest_parameters: &ContestParameters,
    commitment_key: &VoterProofCommitment,
    report: &ProblemReport,
) {
    match choice {
        Choices::Encrypted {
            vote,
            row_proof: Some(proof),
        } => {
            if !verify_voter_proof(
                vote.clone(),
                contest_parameters.election_public_key(),
                commitment_key,
                proof,
            ) {
                report.functional_validation(
                    &format!("Failed to verify proof related to {proposal_ref}"),
                    "'Contest Ballot' document validation",
                );
            }

            if vote.n_options() != contest_parameters.options().n_options() {
                report.invalid_value(
                    "encrypted choices", 
                    &vote.n_options().to_string(),
                    &contest_parameters.options().n_options().to_string(),
                    "'Contest Ballot' must be aligned on contest choices with the 'Contest Parameters'"
                );
            }
        },
        Choices::Clear(choices) => {
            if choices.len() != contest_parameters.options().n_options() {
                report.invalid_value(
                    "clear choices", 
                    &choices.len().to_string(),
                    &contest_parameters.options().n_options().to_string(),
                    "'Contest Ballot' must be aligned on contest choices with the 'Contest Parameters'"
                );
            }
        },
        Choices::Encrypted {
            row_proof: None, ..
        } => {
            report.missing_field(
                "row_proof",
                "'Contest Ballot' must have a proof for an encrypted choice",
            );
        },
    }
}

/// Returns a commitment key calculated from the document reference of the given contest
/// parameters document.
pub(crate) fn commitment_key(doc_ref: &DocumentRef) -> anyhow::Result<VoterProofCommitment> {
    let mut buffer = Vec::new();
    doc_ref
        .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
        .context("Cannot encode 'DocumentRef' for Commitment Key calculation")?;
    let mut hasher = Blake2b512Hasher::new();
    hasher.update(&buffer);
    Ok(VoterProofCommitment::from_hash(hasher))
}
