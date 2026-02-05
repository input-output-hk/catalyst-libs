use std::collections::HashMap;

use catalyst_signed_doc::{
    DocumentRef,
    catalyst_id::{CatalystId, role_index::RoleId},
    tests_utils::{create_dummy_key_pair, create_key_pair_and_publish},
};
use catalyst_voting::vote_protocol::committee::ElectionSecretKey;
use proptest::{
    prelude::{ProptestConfig, prop::array},
    property_test,
};

use crate::tally::{
    contest_tally,
    provider::tests::TestTallyProvider,
    tests::{
        PROPOSALS_AMOUNT, VOTING_OPTIONS, Voter, assert_contest_tally_result, expected_tally,
        prepare_contest, publish_ballot,
    },
};

#[property_test(config = ProptestConfig::with_cases(1))]
fn double_vote_test(
    voting_power: u32,
    #[strategy = array::uniform(0..VOTING_OPTIONS)] first_choices: [usize; PROPOSALS_AMOUNT],
    #[strategy = array::uniform(0..VOTING_OPTIONS)] second_choices: [usize; PROPOSALS_AMOUNT],
    anonymous: bool,
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

    let (sk, kid) = create_key_pair_and_publish(&mut p.p, || create_dummy_key_pair(RoleId::Role0));
    let voter = Voter {
        voting_power,
        anonymous,
        choices: first_choices,
    };
    let sk = sk.into();
    publish_ballot(
        (&voter, kid.clone(), &sk),
        &contest_parameters,
        &proposals_refs,
        &mut p,
    )
    .unwrap();
    // double vote, which must superceed the previous one
    std::thread::sleep(std::time::Duration::from_secs(1));
    let voter = Voter {
        voting_power,
        anonymous,
        choices: second_choices,
    };
    let ballot = publish_ballot(
        (&voter, kid.clone(), &sk),
        &contest_parameters,
        &proposals_refs,
        &mut p,
    )
    .unwrap();

    let exp_tally = expected_tally(&contest_parameters, &proposals_refs, &[voter]);
    let exp_participants: HashMap<CatalystId, (u64, DocumentRef)> =
        [(kid, (voting_power.into(), ballot.doc_ref().unwrap()))]
            .into_iter()
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
