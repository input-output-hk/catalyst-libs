//! Tally proof generation and verification procedures.
//! It allows to transparently verify the correctness of decryption tally procedure.

use std::ops::Mul;

use super::EncryptedTally;
use crate::{
    crypto::{
        group::{GroupElement, Scalar},
        rng::{default_rng, rand_core::CryptoRngCore},
        zk_dl_equality::{DleqProof, generate_dleq_proof, verify_dleq_proof},
    },
    vote_protocol::committee::{ElectionPublicKey, ElectionSecretKey},
};

/// Tally proof struct.
#[allow(clippy::module_name_repetitions)]
#[must_use]
#[derive(Debug, Clone)]
pub struct TallyProof(DleqProof);

/// Generates a tally proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#tally-proof)
#[allow(clippy::module_name_repetitions)]
pub fn generate_tally_proof<R: CryptoRngCore>(
    encrypted_tally: &EncryptedTally,
    secret_key: &ElectionSecretKey,
    rng: &mut R,
) -> TallyProof {
    let randomness = Scalar::random(rng);
    let e1 = encrypted_tally.0.first();
    let d = e1.mul(&secret_key.0);

    let proof = generate_dleq_proof(
        &GroupElement::GENERATOR,
        e1,
        &secret_key.public_key().0,
        &d,
        &secret_key.0,
        &randomness,
    );

    TallyProof(proof)
}

/// Generates a tally proof with `crypto::default_rng`.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#tally-proof)
#[allow(clippy::module_name_repetitions)]
pub fn generate_tally_proof_with_default_rng(
    encrypted_tally: &EncryptedTally,
    secret_key: &ElectionSecretKey,
) -> TallyProof {
    generate_tally_proof(encrypted_tally, secret_key, &mut default_rng())
}

/// Verifies a tally proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#tally-proof)
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn verify_tally_proof(
    encrypted_tally: &EncryptedTally,
    tally: u64,
    public_key: &ElectionPublicKey,
    proof: &TallyProof,
) -> bool {
    let tally = Scalar::from(tally);
    let e1 = encrypted_tally.0.first();
    let e2 = encrypted_tally.0.second();
    let d = e2 - &GroupElement::GENERATOR.mul(&tally);

    verify_dleq_proof(&proof.0, &GroupElement::GENERATOR, e1, &public_key.0, &d)
}
