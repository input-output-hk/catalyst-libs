//! Tally proof generation and verification procedures.
//! It allows to transparently verify the correctness of decryption tally procedure.

use std::ops::Mul;

use rand_core::CryptoRngCore;

use super::EncryptedTally;
use crate::{
    crypto::{
        group::{GroupElement, Scalar},
        zk_dl_equality::{generate_dleq_proof, verify_dleq_proof, DleqProof},
    },
    PublicKey, SecretKey,
};

/// Tally proof struct.
#[allow(clippy::module_name_repetitions)]
pub struct TallyProof(DleqProof);

/// Generates a tally proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#tally-proof)
#[allow(clippy::module_name_repetitions)]
pub fn generate_tally_proof<R: CryptoRngCore>(
    encrypted_tally: &EncryptedTally, secret_key: &SecretKey, rng: &mut R,
) -> TallyProof {
    let randomness = Scalar::random(rng);
    let e1 = encrypted_tally.0.first();
    let d = e1.mul(secret_key);

    let proof = generate_dleq_proof(
        &GroupElement::GENERATOR,
        e1,
        &secret_key.public_key(),
        &d,
        secret_key,
        &randomness,
    );

    TallyProof(proof)
}

/// Verifies a tally proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#tally-proof)
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn verify_tally_proof(
    encrypted_tally: &EncryptedTally, tally: u64, public_key: &PublicKey, proof: &TallyProof,
) -> bool {
    let tally = Scalar::from(tally);
    let e1 = encrypted_tally.0.first();
    let e2 = encrypted_tally.0.second();
    let d = &GroupElement::GENERATOR.mul(&tally) - e2;

    verify_dleq_proof(&proof.0, &GroupElement::GENERATOR, e1, public_key, &d)
}
