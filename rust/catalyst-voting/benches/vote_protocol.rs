//! `catalyst_voting::vote_protocol` benchmark
#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::similar_names
)]

use catalyst_voting::vote_protocol::{
    committee::{ElectionPublicKey, ElectionSecretKey},
    voter::{
        encrypt_vote_with_default_rng,
        proof::{generate_voter_proof_with_default_rng, verify_voter_proof, VoterProofCommitment},
        Vote,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};
use proptest::{
    prelude::{any, Strategy},
    strategy::ValueTree,
    test_runner::TestRunner,
};
use test_strategy::Arbitrary;

const VOTING_OPTIONS: usize = 3;
const VOTERS_NUMBER: usize = 100;

#[derive(Arbitrary, Debug)]
struct Voter {
    _voting_power: u32,
    #[strategy(0..VOTING_OPTIONS)]
    choice: usize,
}

fn initial_setup() -> (
    [Voter; VOTERS_NUMBER],
    ElectionSecretKey,
    ElectionPublicKey,
    VoterProofCommitment,
) {
    let mut runner = TestRunner::default();

    let voters = any::<[Voter; VOTERS_NUMBER]>()
        .new_tree(&mut runner)
        .unwrap()
        .current();

    let election_secret_key = ElectionSecretKey::random_with_default_rng();
    let voter_proof_commitment = VoterProofCommitment::random_with_default_rng();
    let election_public_key = election_secret_key.public_key();

    (
        voters,
        election_secret_key,
        election_public_key,
        voter_proof_commitment,
    )
}

fn vote_protocol_benches(c: &mut Criterion) {
    let (voters, _election_secret_key, election_public_key, voter_proof_commitment) =
        initial_setup();

    let votes: Vec<_> = voters
        .iter()
        .map(|voter| Vote::new(voter.choice, VOTING_OPTIONS).unwrap())
        .collect();

    let mut group = c.benchmark_group("vote protocol benchmark");

    let mut encrypted_votes = Vec::new();
    let mut randomness = Vec::new();
    group.bench_function("vote encryption", |b| {
        b.iter(|| {
            (encrypted_votes, randomness) = votes
                .iter()
                .map(|vote| encrypt_vote_with_default_rng(vote, &election_public_key))
                .unzip();
        });
    });

    let mut voter_proofs = Vec::new();
    group.bench_function("voter proof generation", |b| {
        b.iter(|| {
            voter_proofs = votes
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
        });
    });

    group.bench_function("voter proof verification", |b| {
        b.iter(|| {
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
        });
    });

    group.finish();
}

criterion_group!(benches, vote_protocol_benches);

criterion_main!(benches);
