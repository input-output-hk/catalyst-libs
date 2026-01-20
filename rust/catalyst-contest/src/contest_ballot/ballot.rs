//! An individual Ballot cast in a Contest by a registered user.

use anyhow::{bail, ensure};
use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types::CONTEST_BALLOT, problem_report::ProblemReport,
    providers::CatalystSignedDocumentProvider,
};
use catalyst_voting::crypto::{
    group::GroupElement, hash::Blake2b512Hasher, zk_unit_vector::verify_unit_vector_proof,
};
use minicbor::Decode;

use crate::{
    Choices, ContentBallotPayload, contest_parameters, contest_parameters::ContestParametersPayload,
};

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

        let payload = payload(doc, &report);
        let params = check_parameters(doc, provider, &report)?;
        if let (Some(payload), Some(params)) = (&payload, &params) {
            check_proof(payload, params, &report);
        }

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

    let Ok(payload) = ContentBallotPayload::decode(&mut decoder, &mut report.clone()) else {
        report.functional_validation(
            "Invalid document content: unable to decode CBOR",
            "Cannot get a document content during Contest Ballot document validation.",
        );
        return None;
    };

    Some(payload)
}

/// Checks the parameters of a document and returns a payload.
pub fn check_parameters(
    doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<Option<ContestParametersPayload>> {
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
        return Ok(None);
    };

    if !ContestParameters::timeline_check(doc_ver, &contest_parameters, report, "Contest Ballot") {
        return Ok(());
    }

    Ok(Some(contest_parameters_payload))
}

/// Checks the proof.
pub fn check_proof(
    payload: &ContentBallotPayload,
    params: &ContestParametersPayload,
    report: &ProblemReport,
) {
    for (index, choice) in &payload.choices {
        let Choices::Encrypted {
            choices,
            row_proof: Some(proof),
        } = choice
        else {
            continue;
        };

        // TODO: FIXME:
        let hasher = Blake2b512Hasher::new();
        let commitment_key = GroupElement::from_hash(hasher);

        if !verify_unit_vector_proof(
            proof,
            choices.clone(),
            &params.election_public_key,
            &commitment_key,
        ) {
            report.functional_validation(
                &format!("Failed to verify proof ({index} index)"),
                "'Contest Ballot' document validation",
            );
        }
    }
}
