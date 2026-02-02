//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, builder,
    catalyst_id::role_index::RoleId,
    doc_types,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_verify_and_publish,
        contest_parameters::contest_parameters_default_content, contest_parameters_doc,
        contest_parameters_form_template_doc, create_dummy_admin_key_pair, create_dummy_key_pair,
        create_key_pair_and_publish, proposal_doc, proposal_form_template_doc,
    },
    validator::Validator,
};
use chrono::{Duration, Utc};
use test_case::test_case;

use crate::{
    contest_ballot::{
        ContestBallot, commitment_key,
        payload::{Choices, ContestBallotPayload},
        rule::ContestBallotRule,
    },
    contest_parameters::ContestParameters,
    vote_protocol::voter::proof::VoterProofCommitment,
};

#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let choice = Choices::new_clear_single(0, parameters.options().n_options())?;
        let payload = ContestBallotPayload::new(vec![choice]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => true
    ;
    "valid document, clear choices payload"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let payload = encrypted_payload(&parameters);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => true
    ;
    "valid document, encrypted choices payload"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(1)));
        content["end"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(5)));
        let parameters = build_verify_and_publish(p, |_| builder::contest_parameters_doc(&template.doc_ref()?, &brand.doc_ref()?, &content, &sk.into(), kid, None))?;

        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let choice = Choices::new_clear_single(0, parameters.options().n_options())?;
        let payload = ContestBallotPayload::new(vec![choice]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "failed timeline check, too early"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let choice = Choices::new_clear_single(0, parameters.options().n_options().saturating_add(1))?;
        let payload = ContestBallotPayload::new(vec![choice]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "wrong number of options for clear choice"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let _proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let choice = Choices::new_clear_single(0, parameters.options().n_options())?;
        let payload = ContestBallotPayload::new(vec![choice]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "invalid 'ref' field, does not align with the associated 'Contest Parameters' proposals"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let choice = Choices::Clear((0..parameters.options().n_options()).map(u64::try_from).collect::<Result<Vec<_>, _>>()?);
        let payload = ContestBallotPayload::new(vec![choice]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "not a single clear choice ballot"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(5)));
        content["end"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(1)));
        let parameters = build_verify_and_publish(p, |_| builder::contest_parameters_doc(&template.doc_ref()?, &brand.doc_ref()?, &content, &sk.into(), kid, None))?;

        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let payload = ContestBallotPayload::new(vec![Choices::new_clear_single(0, parameters.options().n_options())?]);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "failed timeline check, too old"
)]
#[test_case(
    |p| {
        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let payload = empty_proof_payload(&parameters);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
    }
    => false
    ;
    "missing proof"
)]
#[test_case(
    |p| {

        let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_verify_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;
        let proposal = build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?;


        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let parameters = ContestParameters::new(&parameters, p)?;
        let payload = invalid_proof_payload(&parameters);
        builder::contest_ballot_doc(&[proposal.doc_ref()?], parameters.doc_ref(), &payload, &sk.into(), kid, None)
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

/// Constructs an encoded payload with encrypted choices
fn encrypted_payload(parameters: &ContestParameters) -> ContestBallotPayload {
    let commitment = commitment_key(parameters.doc_ref()).unwrap();
    let choice = Choices::new_encrypted_single(
        1,
        parameters.options().n_options(),
        parameters.election_public_key(),
        &commitment,
    )
    .unwrap();
    ContestBallotPayload::new(vec![choice])
}

/// Constructs an encoded payload with encrypted choices, but without proof.
fn empty_proof_payload(parameters: &ContestParameters) -> ContestBallotPayload {
    let commitment = commitment_key(parameters.doc_ref()).unwrap();
    let mut choice = Choices::new_encrypted_single(
        1,
        parameters.options().n_options(),
        parameters.election_public_key(),
        &commitment,
    )
    .unwrap();
    if let Choices::Encrypted { row_proof, .. } = &mut choice {
        *row_proof = None;
    }
    ContestBallotPayload::new(vec![choice])
}

/// Constructs an encoded payload with encrypted choices, but with an invalid proof.
fn invalid_proof_payload(parameters: &ContestParameters) -> ContestBallotPayload {
    let wrong_commitment = VoterProofCommitment::random_with_default_rng();
    let choice = Choices::new_encrypted_single(
        1,
        parameters.options().n_options(),
        parameters.election_public_key(),
        &wrong_commitment,
    )
    .unwrap();
    ContestBallotPayload::new(vec![choice])
}
