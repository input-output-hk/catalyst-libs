//! Voter proof generation and verification procedures.
//! It allows to transparently verify the correctness voter generation and encryption.

use std::ops::Mul;

use rand_core::CryptoRngCore;

use super::{EncryptedVote, EncryptionRandomness, Vote};
use crate::{
    crypto::{
        group::{GroupElement, Scalar},
        zk_unit_vector::{generate_unit_vector_proof, verify_unit_vector_proof, UnitVectorProof},
    },
    PublicKey,
};

/// Tally proof struct.
#[allow(clippy::module_name_repetitions)]
pub struct VoterProof(UnitVectorProof);

/// Voter proof commitment struct.
pub struct VoterProofCommitment(GroupElement);

impl VoterProofCommitment {
    /// Randomly generate the `VoterProofCommitment`.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self(GroupElement::GENERATOR.mul(&Scalar::random(rng)))
    }
}

/// Generate voter proof error
#[derive(thiserror::Error, Debug)]
pub enum GenerateVoterProofError {
    /// Arguments mismatch
    #[error("Provided arguments mismatch. Size of the provided `vote`: {0}, `encrypted_vote: {1}` and `randomness`: {2} must be equal with each other.")]
    ArgumentsMismatch(usize, usize, usize),
}

/// Generates a voter proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
///
/// # Errors
///   - `GenerateVoterProofError`
#[allow(clippy::module_name_repetitions)]
pub fn generate_voter_proof<R: CryptoRngCore>(
    vote: &Vote, encrypted_vote: EncryptedVote, randomness: EncryptionRandomness,
    public_key: &PublicKey, commitment: &VoterProofCommitment, rng: &mut R,
) -> Result<VoterProof, GenerateVoterProofError> {
    if vote.voting_options != encrypted_vote.0.len() || vote.voting_options != randomness.0.len() {
        return Err(GenerateVoterProofError::ArgumentsMismatch(
            vote.voting_options,
            encrypted_vote.0.len(),
            randomness.0.len(),
        ));
    }

    let proof = generate_unit_vector_proof(
        &vote.to_unit_vector(),
        encrypted_vote.0,
        randomness.0,
        public_key,
        &commitment.0,
        rng,
    );
    Ok(VoterProof(proof))
}

/// Verifies a voter proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn verify_voter_proof(
    encrypted_vote: EncryptedVote, public_key: &PublicKey, commitment: &VoterProofCommitment,
    proof: &VoterProof,
) -> bool {
    verify_unit_vector_proof(&proof.0, encrypted_vote.0, public_key, &commitment.0)
}
