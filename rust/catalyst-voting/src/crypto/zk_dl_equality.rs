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

use curve25519_dalek::digest::Update;

use super::{
    group::{GroupElement, Scalar},
    hash::Blake2b512Hasher,
};

/// DLEQ proof struct
pub struct DleqProof(Scalar, Scalar);

/// Generates a DLEQ proof.
pub fn generate_dleq_proof(
    base_1: &GroupElement, base_2: &GroupElement, point_1: &GroupElement, point_2: &GroupElement,
    dlog: &Scalar, randomness: &Scalar,
) -> DleqProof {
    let a_1 = base_1 * randomness;
    let a_2 = base_2 * randomness;

    let blake2b_hasher = Blake2b512Hasher::new()
        .chain(a_1.to_bytes())
        .chain(a_2.to_bytes())
        .chain(point_1.to_bytes())
        .chain(point_2.to_bytes());

    let challenge = Scalar::from_hash(blake2b_hasher);
    let response = &(dlog * &challenge) + randomness;

    DleqProof(challenge, response)
}
