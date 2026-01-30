use std::collections::HashMap;

use catalyst_signed_doc::{
    DocumentRef, builder,
    catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_doc_and_publish,
        contest_parameters::contest_parameters_default_content,
        contest_parameters_form_template_doc, create_dummy_key_pair, create_key_pair_and_publish,
        proposal_doc, proposal_form_template_doc,
    },
};
use catalyst_voting::vote_protocol::committee::{ElectionPublicKey, ElectionSecretKey};
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
    tally::{provider::tests::TestTallyProvider, tally},
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

    for voter in voters {
        publish_ballot(&voter, &contest_parameters, &proposals_refs, &mut p).unwrap();
    }

    let res_tally = tally(&contest_parameters, &election_secret_key, &p).unwrap();
    assert_eq!(&res_tally.options, contest_parameters.options());

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
            assert_eq!(
                res_tally_per_proposal[i].decrypted_tally,
                exp_tally_per_proposal[i].1
            );
            assert_eq!(
                res_tally_per_proposal[i].clear_tally,
                exp_tally_per_proposal[i].0
            );
        }
    }
}

fn prepare_contest(
    options: &[String; VOTING_OPTIONS],
    proposals_amount: usize,
    election_public_key: &ElectionPublicKey,
    p: &mut TestCatalystProvider,
) -> anyhow::Result<(ContestParameters, Vec<DocumentRef>)> {
    let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));

    let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
    let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
    let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;

    let mut content = contest_parameters_default_content();
    content["election_public_key"] = serde_json::json!(hex::encode(election_public_key.to_bytes()));
    content["options"] = serde_json::json!(options);
    let parameters = build_doc_and_publish(p, |_| {
        builder::contest_parameters_doc(
            &template.doc_ref()?,
            &brand.doc_ref()?,
            &content,
            &sk.clone().into(),
            kid.clone(),
            None,
        )
    })?;
    let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&brand, p))?;

    let proposals_refs = (0..proposals_amount)
        .map(|_| build_doc_and_publish(p, |p| proposal_doc(&template, &brand, p))?.doc_ref())
        .collect::<Result<_, _>>()?;

    Ok((ContestParameters::new(&parameters, p)?, proposals_refs))
}

fn publish_ballot(
    voter: &Voter,
    parameters: &ContestParameters,
    proposals_refs: &[DocumentRef],
    p: &mut TestTallyProvider,
) -> anyhow::Result<()> {
    let (sk, kid) = create_key_pair_and_publish(&mut p.p, || create_dummy_key_pair(RoleId::Role0));

    // Filling the `TestTallyProvider` with the voter's voting power information
    p.voters.insert(kid.clone(), voter.voting_power.into());

    let choices = voter
        .choices
        .iter()
        .map(|choice| {
            if voter.anonymous {
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

    build_doc_and_publish(&mut p.p, |_| {
        builder::contest_ballot_doc(
            proposals_refs,
            parameters.doc_ref(),
            &payload,
            &sk.into(),
            kid,
            None,
        )
    })?;

    Ok(())
}

type EncryptedTotalResult = u64;
type ClearTotalResult = u64;

fn expected_tally(
    contest_parameters: &ContestParameters,
    proposals_refs: &[DocumentRef],
    voters: &[Voter],
) -> HashMap<DocumentRef, Vec<(ClearTotalResult, EncryptedTotalResult, String)>> {
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
