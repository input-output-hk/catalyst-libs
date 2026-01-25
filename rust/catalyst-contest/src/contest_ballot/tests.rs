//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, builder,
    catalyst_id::role_index::RoleId,
    doc_types,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_doc_and_publish,
        contest_ballot_doc, contest_parameters::contest_parameters_default_content,
        contest_parameters_doc, contest_parameters_form_template_doc, create_dummy_key_pair,
        create_key_pair_and_publish, proposal_doc, proposal_form_template_doc,
    },
    validator::Validator,
};
use catalyst_voting::{
    crypto::group::GroupElement,
    vote_protocol::voter::{
        EncryptedVote, Vote, encrypt_vote_with_default_rng,
        proof::{VoterProof, VoterProofCommitment, generate_voter_proof_with_default_rng},
    },
};
use chrono::{Duration, Utc};
use minicbor::{Encode, Encoder};
use test_case::test_case;

use crate::contest_ballot::{
    ContestBallot,
    payload::{Choices, ContestBallotPayload},
    rule::ContestBallotRule,
};

#[test_case(
    |p| {
        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let payload = clear_payload();

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        builder::contest_ballot_doc(&proposal, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None, &payload)
    }
    => true
    ;
    "valid document, clear payload"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(1)));
        content["end"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(5)));

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |_| builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => false
    ;
    "failed timeline check, too early"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(5)));
        content["end"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(1)));

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |_| builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => false
    ;
    "failed timeline check, too old"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let payload = empty_proof_payload();

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        builder::contest_ballot_doc(&proposal, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None, &payload)
    }
    => false
    ;
    "missing proof"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let payload = invalid_proof_payload();

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        builder::contest_ballot_doc(&proposal, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None, &payload)
    }
    => false
    ;
    "invalid proof"
)]
fn contest_ballot(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator.extend_rules_per_document(doc_types::CONTEST_BALLOT.clone(), ContestBallotRule {});

    validator.validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    let is_valid = !doc.report().is_problematic();

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report.
    let doc = doc_gen(&mut provider).unwrap();
    let contest_ballot = ContestBallot::new(&doc, &provider).unwrap();
    println!("{:?}", contest_ballot.report());
    assert_eq!(is_valid, !contest_ballot.report().is_problematic());

    is_valid
}

/// Constructs an encoded payload with encrypted choices, but without proof.
fn empty_proof_payload() -> Vec<u8> {
    let vote = Vote::new(0, 1).unwrap();
    let key = GroupElement::zero().into();
    let vote = encrypt_vote_with_default_rng(&vote, &key).0;
    encode_encrypted_payload(vote, None)
}

/// Constructs an encoded payload with encrypted choices, but with an invalid proof.
fn invalid_proof_payload() -> Vec<u8> {
    let vote = Vote::new(0, 1).unwrap();
    let key = GroupElement::zero().into();
    let (encrypted_vote, randomness) = encrypt_vote_with_default_rng(&vote, &key);
    let public_key = GroupElement::zero().into();
    let commitment = VoterProofCommitment::random_with_default_rng();
    let proof = generate_voter_proof_with_default_rng(
        &vote,
        encrypted_vote.clone(),
        randomness,
        &public_key,
        &commitment,
    )
    .unwrap();
    encode_encrypted_payload(encrypted_vote, Some(proof))
}

/// Constructs a payload with clear choices.
fn clear_payload() -> Vec<u8> {
    let choices = [(0, Choices::Clear(vec![0, 1, 2]))]
        .iter()
        .cloned()
        .collect();
    let payload = ContestBallotPayload {
        choices,
        column_proof: None,
        matrix_proof: None,
        voter_choices: None,
    };
    encode_payload(&payload)
}

/// Encodes an encrypted contest ballot payload.
fn encode_encrypted_payload(
    vote: EncryptedVote,
    row_proof: Option<VoterProof>,
) -> Vec<u8> {
    let choices = [(0, Choices::Encrypted { vote, row_proof })]
        .iter()
        .cloned()
        .collect();
    let payload = ContestBallotPayload {
        choices,
        column_proof: None,
        matrix_proof: None,
        voter_choices: None,
    };
    encode_payload(&payload)
}

/// Encodes the given contest ballot payload.
fn encode_payload(payload: &ContestBallotPayload) -> Vec<u8> {
    let mut buffer = Vec::new();
    payload
        .encode(&mut Encoder::new(&mut buffer), &mut ())
        .unwrap();
    buffer
}
