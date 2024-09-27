//! Voting primitives which are used among Catalyst ecosystem.

#![allow(dead_code, unused_variables, clippy::todo)]

mod crypto;

/// A representation of the voting choice.
pub struct Vote;

/// Generate a vote.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voting-choice)
///
/// # Errors
///   - TODO
pub fn vote(vote: usize, voting_options: usize) -> anyhow::Result<Vote> {
    todo!()
}

/// A representation of the encrypted vote.
pub struct EncryptedVote;

/// Election public key.
pub struct ElectionPublicKey;

/// Encrypt vote function.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#vote-encryption)
///
/// # Errors
///   - TODO
pub fn encrypt_vote(vote: &Vote, election_pk: &ElectionPublicKey) -> anyhow::Result<EncryptedVote> {
    todo!()
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
