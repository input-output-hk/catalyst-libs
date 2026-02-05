mod double_vote;

use std::collections::HashMap;

use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef,
    builder::{self, ed25519::Ed25519SigningKey},
    catalyst_id::{CatalystId, role_index::RoleId},
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_verify_and_publish,
        contest_parameters::contest_parameters_default_content,
        contest_parameters_form_template_doc, create_dummy_admin_key_pair, create_dummy_key_pair,
        create_key_pair_and_publish, proposal_doc, proposal_form_template_doc,
    },
};
use catalyst_voting::vote_protocol::{
    committee::{ElectionPublicKey, ElectionSecretKey},
    tally::proof::verify_tally_proof,
};
use proptest::{
    prelude::{Just, ProptestConfig, prop::array},
    property_test,
};
use proptest_derive::Arbitrary;

use crate::{
    contest_ballot::{
        commitment_key,
        payload::{Choices, ContestBallotPayload},
    },
    contest_parameters::ContestParameters,
    tally::{ContestResult, contest_tally, provider::tests::TestTallyProvider},
};

const VOTING_OPTIONS: usize = 3;
const VOTERS_NUMBER: usize = 10;
const PROPOSALS_AMOUNT: usize = 10;

#[derive(Arbitrary, Debug, Clone)]
struct Voter {
    voting_power: u32,
    #[proptest(strategy = "array::uniform(0..VOTING_OPTIONS)")]
    choices: [usize; PROPOSALS_AMOUNT],
    #[proptest(strategy = "Just(false)")]
    anonymous: bool,
}

type EncryptedTotalResult = u64;
type ClearTotalResult = u64;
type Participants = HashMap<CatalystId, (u64, DocumentRef)>;
type TallyRes = HashMap<DocumentRef, Vec<(ClearTotalResult, EncryptedTotalResult, String)>>;

#[property_test(config = ProptestConfig::with_cases(1))]
fn tally_test(
    voters: [Voter; VOTERS_NUMBER],
    options: [String; VOTING_OPTIONS],
) {
    let election_secret_key = ElectionSecretKey::random_with_default_rng();
    let mut p = TestTallyProvider::default();

    let (contest_parameters, proposals_refs) = prepare_contest(
        &options,
        PROPOSALS_AMOUNT,
        &election_secret_key.public_key(),
        &mut p.p,
    )
    .unwrap();
    assert_eq!(proposals_refs.len(), PROPOSALS_AMOUNT);

    let exp_tally = expected_tally(&contest_parameters, &proposals_refs, &voters);

    let exp_participants: Participants = voters
        .iter()
        .map(|v| {
            let ballot =
                publish_ballot_with_keys(v, &contest_parameters, &proposals_refs, &mut p).unwrap();
            let voter = ballot.authors().first().unwrap().clone();
            let ballot_ref = ballot.doc_ref().unwrap();

            (voter, (v.voting_power.into(), ballot_ref))
        })
        .collect();

    let res_tally = contest_tally(&contest_parameters, Some(&election_secret_key), &p).unwrap();
    assert_contest_tally_result(
        &election_secret_key.public_key(),
        res_tally,
        &contest_parameters,
        exp_tally,
        exp_participants,
    );
}

fn assert_contest_tally_result(
    election_public_key: &ElectionPublicKey,
    res_tally: ContestResult,
    contest_parameters: &ContestParameters,
    exp_tally: TallyRes,
    exp_participants: Participants,
) {
    assert_eq!(&res_tally.options, contest_parameters.options());
    assert_eq!(&res_tally.participants, &exp_participants);

    for (p_ref, exp_tally_per_proposal) in exp_tally {
        let res_tally_per_proposal = res_tally
            .tally_per_proposals
            .get(&p_ref)
            .expect("missing tally result for the proposal");

        for i in 0..exp_tally_per_proposal.len() {
            assert_eq!(
                res_tally_per_proposal[i].option,
                exp_tally_per_proposal[i].2
            );
            let decrypted_tally = res_tally_per_proposal[i]
                .decrypted_tally
                .as_ref()
                .expect("must have decrypted tally");
            assert_eq!(decrypted_tally.tally, exp_tally_per_proposal[i].1);
            assert_eq!(
                res_tally_per_proposal[i].clear_tally,
                exp_tally_per_proposal[i].0
            );
            assert!(verify_tally_proof(
                &res_tally_per_proposal[i].encrypted_tally,
                decrypted_tally.tally,
                &election_public_key,
                &decrypted_tally.proof
            ));
        }
    }
}

fn prepare_contest(
    options: &[String; VOTING_OPTIONS],
    proposals_amount: usize,
    election_public_key: &ElectionPublicKey,
    p: &mut TestCatalystProvider,
) -> anyhow::Result<(ContestParameters, Vec<DocumentRef>)> {
    let brand = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
    let brand = build_verify_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
    let template =
        build_verify_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;

    let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
    let mut content = contest_parameters_default_content();
    content["election_public_key"] = serde_json::json!(hex::encode(election_public_key.to_bytes()));
    content["options"] = serde_json::json!(options);
    let parameters = build_verify_and_publish(p, |_| {
        builder::contest_parameters_doc(
            &template.doc_ref()?,
            &brand.doc_ref()?,
            &content,
            &sk.clone().into(),
            kid.clone(),
            None,
        )
    })?;
    let template = build_verify_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;

    let proposals_refs = (0..proposals_amount)
        .map(|_| build_verify_and_publish(p, |p| proposal_doc(&template, &brand, p))?.doc_ref())
        .collect::<Result<_, _>>()?;

    Ok((ContestParameters::new(&parameters, p)?, proposals_refs))
}

fn publish_ballot_with_keys(
    voter: &Voter,
    parameters: &ContestParameters,
    proposals_refs: &[DocumentRef],
    p: &mut TestTallyProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_key_pair_and_publish(&mut p.p, || create_dummy_key_pair(RoleId::Role0));
    publish_ballot((voter, kid, &sk.into()), parameters, proposals_refs, p)
}

fn publish_ballot(
    voter: (&Voter, CatalystId, &Ed25519SigningKey),
    parameters: &ContestParameters,
    proposals_refs: &[DocumentRef],
    p: &mut TestTallyProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    // Filling the `TestTallyProvider` with the voter's voting power information
    p.voters
        .insert(voter.1.clone(), voter.0.voting_power.into());

    let choices = voter
        .0
        .choices
        .iter()
        .map(|choice| {
            if voter.0.anonymous {
                let commitment = commitment_key(parameters.doc_ref())?;
                Choices::new_encrypted_single(
                    *choice,
                    parameters.options().n_options(),
                    parameters.election_public_key(),
                    &commitment,
                )
            } else {
                Choices::new_clear_single(*choice, parameters.options().n_options())
            }
        })
        .collect::<Result<_, _>>()?;
    let payload = ContestBallotPayload::new(choices);

    build_verify_and_publish(&mut p.p, |_| {
        builder::contest_ballot_doc(
            proposals_refs,
            parameters.doc_ref(),
            &payload,
            voter.2,
            voter.1,
            None,
        )
    })
}

fn expected_tally(
    contest_parameters: &ContestParameters,
    proposals_refs: &[DocumentRef],
    voters: &[Voter],
) -> TallyRes {
    let options = contest_parameters.options().clone();

    proposals_refs
        .iter()
        .enumerate()
        .map(|(p_index, p_ref)| {
            let res = options
                .iter()
                .enumerate()
                .map(|(option_index, option)| {
                    // collects a voting result for a proposal per voting option
                    let p_clear_result_per_option = voters
                        .iter()
                        // filters ALL encrypted option, to keep only clear one
                        .filter(|v| !v.anonymous)
                        // filters each voter's choice that its done exactly on the `option_index`
                        .filter(|v| {
                            let choice = v.choices.get(p_index).expect("missing proposal choice");
                            choice == &option_index
                        })
                        // take filtered voting power
                        .map(|v| u64::from(v.voting_power))
                        .sum();

                    let p_encrypted_result_per_option = voters
                        .iter()
                        // filters ALL clear option, to keep only encrypted one
                        .filter(|v| v.anonymous)
                        // filters each voter's choice that its done exactly on the `option_index`
                        .filter(|v| {
                            let choice = v.choices.get(p_index).expect("missing proposal choice");
                            choice == &option_index
                        })
                        // take filtered voting power
                        .map(|v| u64::from(v.voting_power))
                        .sum();

                    (
                        p_clear_result_per_option,
                        p_encrypted_result_per_option,
                        option.clone(),
                    )
                })
                .collect::<Vec<_>>();

            (p_ref.clone(), res)
        })
        .collect()
}
