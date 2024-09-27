//! Voting primitives which are used among Catalyst ecosystem.

#![allow(dead_code, unused_variables, clippy::todo)]

use voter::{EncryptedVote, EncryptedVoteError, EncryptionRandomness, Vote, VoteError};

mod crypto;
pub mod tally;
pub mod voter;

/// Election public key.
pub type ElectionPublicKey = crypto::elgamal::PublicKey;

/// Generate a vote.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voting-choice)
///
/// # Errors
///   - `voter::VoteError`
pub fn vote(choice: usize, voting_options: usize) -> Result<Vote, VoteError> {
    Vote::new(choice, voting_options)
}

/// Encrypt vote function.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#vote-encryption)
///
/// # Errors
///   - `voter::EncryptedVoteError`
pub fn encrypt_vote(
    vote: &Vote, election_pk: &ElectionPublicKey, randomness: &EncryptionRandomness,
) -> Result<EncryptedVote, EncryptedVoteError> {
    vote.encrypt(election_pk, randomness)
}

/// Vote proof struct.
pub struct VoteProof;

/// Generates a vote proof, which proofs that the given encrypted vote was correctly
/// generated.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
///
/// # Errors
///   - TODO
pub fn generate_vote_proof(
    vote: &Vote, encrypted_vote: &EncryptedVote, election_pk: &ElectionPublicKey,
) -> anyhow::Result<VoteProof> {
    todo!()
}

/// Verifies a vote proof, is it valid or not for the given encrypted vote.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
///
/// # Errors
///   - TODO
pub fn verify_vote(
    vote: &Vote, proof: &VoteProof, election_pk: &ElectionPublicKey,
) -> anyhow::Result<()> {
    todo!()
}
