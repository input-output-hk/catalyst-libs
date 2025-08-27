//! A general voting integration test, which performs a full voting procedure.
#![allow(clippy::explicit_deref_methods)]

use catalyst_voting::vote_protocol::{
    committee::ElectionSecretKey,
    tally::{
        decrypt_tally,
        proof::{generate_tally_proof_with_default_rng, verify_tally_proof},
        tally, DecryptionTallySetup,
    },
    voter::{
        decrypt_vote, encrypt_vote_with_default_rng,
        proof::{generate_voter_proof_with_default_rng, verify_voter_proof, VoterProofCommitment},
        Vote,
    },
};
use proptest::prelude::ProptestConfig;
use test_strategy::{proptest, Arbitrary};

const VOTING_OPTIONS: usize = 3;
const VOTERS_NUMBER: usize = 100;

#[derive(Arbitrary, Debug)]
struct Voter {
    voting_power: u32,
    #[strategy(0..VOTING_OPTIONS)]
    choice: usize,
}

#[proptest(ProptestConfig::with_cases(1))]
fn voting_test(voters: [Voter; VOTERS_NUMBER]) {
    let election_secret_key = ElectionSecretKey::random_with_default_rng();
    let election_public_key = election_secret_key.public_key();
    let voter_proof_commitment = VoterProofCommitment::random_with_default_rng();

    let votes: Vec<_> = voters
        .iter()
        .map(|voter| Vote::new(voter.choice, VOTING_OPTIONS).unwrap())
        .collect();

    let (encrypted_votes, randomness): (Vec<_>, Vec<_>) = votes
        .iter()
        .map(|vote| encrypt_vote_with_default_rng(vote, &election_public_key))
        .unzip();

    // Decrypting votes
    {
        let decrypted_votes: Vec<_> = encrypted_votes
            .iter()
            .map(|v| decrypt_vote(v, &election_secret_key).unwrap())
            .collect();
        assert_eq!(votes, decrypted_votes);
    }

    // Verify encrypted votes
    {
        let voter_proofs: Vec<_> = votes
            .iter()
            .zip(encrypted_votes.iter())
            .zip(randomness.iter())
            .map(|((v, enc_v), r)| {
                generate_voter_proof_with_default_rng(
                    v,
                    enc_v.clone(),
                    r.clone(),
                    &election_public_key,
                    &voter_proof_commitment,
                )
                .unwrap()
            })
            .collect();

        let is_ok = voter_proofs
            .iter()
            .zip(encrypted_votes.iter())
            .all(|(p, enc_v)| {
                verify_voter_proof(
                    enc_v.clone(),
                    &election_public_key,
                    &voter_proof_commitment,
                    p,
                )
            });
        assert!(is_ok);
    }

    let voting_powers: Vec<_> = voters
        .iter()
        .map(|voter| u64::from(voter.voting_power))
        .collect();

    let encrypted_tallies: Vec<_> = (0..VOTING_OPTIONS)
        .map(|voting_option| tally(voting_option, &encrypted_votes, &voting_powers).unwrap())
        .collect();

    let total_voting_power = voting_powers.iter().sum();
    let decryption_tally_setup = DecryptionTallySetup::new(total_voting_power).unwrap();

    let decrypted_tallies: Vec<_> = encrypted_tallies
        .iter()
        .map(|t| decrypt_tally(t, &election_secret_key, &decryption_tally_setup).unwrap())
        .collect();

    // Verify tallies
    {
        let tally_proofs: Vec<_> = encrypted_tallies
            .iter()
            .map(|t| generate_tally_proof_with_default_rng(t, &election_secret_key))
            .collect();

        let is_ok = tally_proofs
            .iter()
            .zip(encrypted_tallies.iter())
            .zip(decrypted_tallies.iter())
            .all(|((p, enc_t), t)| verify_tally_proof(enc_t, *t, &election_public_key, p));
        assert!(is_ok);
    }

    let expected_tallies: Vec<_> = (0..VOTING_OPTIONS)
        .map(|i| {
            voters
                .iter()
                .filter(|v| v.choice == i)
                .map(|v| u64::from(v.voting_power))
                .sum::<u64>()
        })
        .collect();

    assert_eq!(decrypted_tallies, expected_tallies);
}
