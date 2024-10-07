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

/// Generates a voter proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
#[allow(clippy::module_name_repetitions)]
pub fn generate_voter_proof<R: CryptoRngCore>(
    vote: &Vote, encrypted_vote: EncryptedVote, randomness: EncryptionRandomness,
    public_key: &PublicKey, commitment: &VoterProofCommitment, rng: &mut R,
) -> VoterProof {
    let proof = generate_unit_vector_proof(
        &vote.to_unit_vector(),
        encrypted_vote.0,
        randomness.0,
        public_key,
        &commitment.0,
        rng,
    );
    VoterProof(proof)
}

/// Verifies a voter proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voters-proof)
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn verify_voter_proof(
    encrypted_vote: EncryptedVote, public_key: &PublicKey, commitment_key: &PublicKey,
    proof: &VoterProof,
) -> bool {
    verify_unit_vector_proof(&proof.0, encrypted_vote.0, public_key, commitment_key)
}
