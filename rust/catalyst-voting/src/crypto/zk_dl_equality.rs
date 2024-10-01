//! Non-interactive Zero Knowledge proof of Discrete Logarithm
//! Equality (DLEQ).
//!
//! The proof is the following:
//!
//! `NIZK{(base_1, base_2, point_1, point_2), (dlog): point_1 = base_1^dlog AND point_2 =
//! base_2^dlog}`
//!
//! which makes the statement, the two bases `base_1` and `base_2`, and the two
//! points `point_1` and `point_2`. The witness, on the other hand
//! is the discrete logarithm, `dlog`.

#![allow(dead_code, unused_variables)]

use super::group::{GroupElement, Scalar};

/// DLEQ proof struct
pub struct DleqProof(Scalar, Scalar);

/// Generates a DLEQ proof.
pub fn generate_dleq_proof(
    base_1: &GroupElement, base_2: &GroupElement, point_1: &GroupElement, point_2: &GroupElement,
    dlog: &Scalar, randomness: &Scalar,
) -> DleqProof {
    let a_1 = base_1 * randomness;
    let a_2 = base_2 * randomness;

    let mut blake2b_hasher = blake2b_simd::State::new();
    blake2b_hasher.update(&a_1.to_bytes());
    blake2b_hasher.update(&a_2.to_bytes());
    blake2b_hasher.update(&point_1.to_bytes());
    blake2b_hasher.update(&point_2.to_bytes());
    let hash = blake2b_hasher.finalize();
    hash.as_bytes();

    let challenge = Scalar::zero();
    let response = &(dlog * &challenge) + randomness;

    DleqProof(challenge, response)
}
