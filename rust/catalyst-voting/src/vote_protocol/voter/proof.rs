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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoterProof(UnitVectorProof);

impl VoterProof {
    /// Decode `VoterProof` from bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8], size: usize) -> Option<Self> {
        UnitVectorProof::from_bytes(bytes, size).map(Self)
    }

    /// Get a deserialized bytes size
    #[must_use]
    pub fn bytes_size(&self) -> usize {
        self.0.bytes_size()
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

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

#[cfg(test)]
mod tests {
    use proptest::prelude::{any_with, Arbitrary, BoxedStrategy, Strategy};

    use super::*;

    impl Arbitrary for VoterProof {
        type Parameters = usize;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(size: Self::Parameters) -> Self::Strategy {
            any_with::<UnitVectorProof>(size).prop_map(Self).boxed()
        }
    }
}
