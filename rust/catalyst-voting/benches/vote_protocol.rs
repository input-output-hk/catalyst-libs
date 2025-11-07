//! `catalyst_voting::vote_protocol` benchmark
//!
//! To run these benchmarks use
//! ```shell
//! SAMPLE_SIZE=<sample size> VOTERS_NUMBER=<voters number> cargo bench -p catalyst-voting vote_protocol
//! ```
#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::similar_names,
    clippy::explicit_deref_methods
)]

use catalyst_voting::{
    crypto::rng::default_rng,
    vote_protocol::{
        committee::{ElectionPublicKey, ElectionSecretKey},
        tally::{
            DecryptionTallySetup, decrypt_tally,
            proof::{generate_tally_proof, verify_tally_proof},
            tally,
        },
        voter::{
            Vote, encrypt_vote,
            proof::{VoterProofCommitment, generate_voter_proof, verify_voter_proof},
        },
    },
};
use criterion::{Criterion, criterion_group, criterion_main};
use proptest::{
    prelude::{Strategy, any_with},
    sample::size_range,
    strategy::ValueTree,
    test_runner::TestRunner,
};
use test_strategy::Arbitrary;

const VOTERS_NUMBER_ENV: &str = "VOTERS_NUMBER";
const SAMPLE_SIZE_ENV: &str = "SAMPLE_SIZE";
const DEFAULT_SAMPLE_SIZE: usize = 10;
const DEFAULT_VOTERS_NUMBER: usize = 1;

const VOTING_OPTIONS: usize = 3;

#[derive(Arbitrary, Debug)]
struct Voter {
    voting_power: u32,
    #[strategy(0..VOTING_OPTIONS)]
    choice: usize,
}

struct Choices(Vec<usize>);
struct VotingPowers(Vec<u64>);

fn rand_generate_vote_data(
    voters_number: usize
) -> (
    Choices,
    VotingPowers,
    ElectionSecretKey,
    ElectionPublicKey,
    VoterProofCommitment,
) {
    let mut runner = TestRunner::default();

    let (choices, voting_powers) = any_with::<Vec<Voter>>((size_range(voters_number), ()))
        .prop_map(|voter| {
            (
                voter.iter().map(|v| v.choice).collect(),
                voter.iter().map(|v| v.voting_power.into()).collect(),
            )
        })
        .new_tree(&mut runner)
        .unwrap()
        .current();

    let election_secret_key = ElectionSecretKey::random_with_default_rng();
    let voter_proof_commitment = VoterProofCommitment::random_with_default_rng();
    let election_public_key = election_secret_key.public_key();

    (
        Choices(choices),
        VotingPowers(voting_powers),
        election_secret_key,
        election_public_key,
        voter_proof_commitment,
    )
}

#[allow(clippy::too_many_lines)]
fn vote_protocol_benches(c: &mut Criterion) {
    let sample_size = std::env::var(SAMPLE_SIZE_ENV)
        .map(|s| s.parse().unwrap())
        .unwrap_or(DEFAULT_SAMPLE_SIZE);
    let voters_number = std::env::var(VOTERS_NUMBER_ENV)
        .map(|s| s.parse().unwrap())
        .unwrap_or(DEFAULT_VOTERS_NUMBER);

    let mut group = c.benchmark_group("vote protocol benchmark");
    group.sample_size(sample_size);

    let (choices, voting_powers, election_secret_key, election_public_key, voter_proof_commitment) =
        rand_generate_vote_data(voters_number);

    let votes: Vec<_> = choices
        .0
        .iter()
        .map(|choice| Vote::new(*choice, VOTING_OPTIONS).unwrap())
        .collect();
    let mut rng = default_rng();

    let mut encrypted_votes = Vec::new();
    let mut randomness = Vec::new();
    group.bench_function("vote encryption", |b| {
        b.iter(|| {
            (encrypted_votes, randomness) = votes
                .iter()
                .map(|vote| encrypt_vote(vote, &election_public_key, &mut rng))
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
                    generate_voter_proof(
                        v,
                        enc_v.clone(),
                        r.clone(),
                        &election_public_key,
                        &voter_proof_commitment,
                        &mut rng,
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

    let mut encrypted_tallies = Vec::new();
    group.bench_function("tally", |b| {
        b.iter(|| {
            encrypted_tallies = (0..VOTING_OPTIONS)
                .map(|voting_option| {
                    tally(voting_option, &encrypted_votes, &voting_powers.0).unwrap()
                })
                .collect();
        });
    });

    let total_voting_power = voting_powers.0.iter().sum();
    let mut decryption_tally_setup = None;
    group.bench_function("decryption tally setup initialization", |b| {
        b.iter(|| {
            decryption_tally_setup = Some(DecryptionTallySetup::new(total_voting_power).unwrap());
        });
    });
    let decryption_tally_setup = decryption_tally_setup.unwrap();

    let mut decrypted_tallies = Vec::new();
    group.bench_function("decrypt tally", |b| {
        b.iter(|| {
            decrypted_tallies = encrypted_tallies
                .iter()
                .map(|t| decrypt_tally(t, &election_secret_key, &decryption_tally_setup).unwrap())
                .collect();
        });
    });

    let mut tally_proofs = Vec::new();
    group.bench_function("tally proof generation", |b| {
        b.iter(|| {
            tally_proofs = encrypted_tallies
                .iter()
                .map(|t| generate_tally_proof(t, &election_secret_key, &mut rng))
                .collect();
        });
    });

    group.bench_function("tally proof verification", |b| {
        b.iter(|| {
            let is_ok = tally_proofs
                .iter()
                .zip(encrypted_tallies.iter())
                .zip(decrypted_tallies.iter())
                .all(|((p, enc_t), t)| verify_tally_proof(enc_t, *t, &election_public_key, p));
            assert!(is_ok);
        });
    });

    group.finish();
}

criterion_group!(benches, vote_protocol_benches);

criterion_main!(benches);
