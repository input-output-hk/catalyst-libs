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

// cspell: words NIZK

use crate::crypto::{
    group::{GroupElement, Scalar},
    hash::{Blake2b512Hasher, digest::Digest},
};

/// DLEQ proof struct
#[must_use]
#[derive(Debug, Clone)]
pub struct DleqProof(Scalar, Scalar);

/// Generates a DLEQ proof.
pub fn generate_dleq_proof(
    base_1: &GroupElement,
    base_2: &GroupElement,
    point_1: &GroupElement,
    point_2: &GroupElement,
    dlog: &Scalar,
    randomness: &Scalar,
) -> DleqProof {
    let a_1 = base_1 * randomness;
    let a_2 = base_2 * randomness;

    let challenge = calculate_challenge(base_1, base_2, point_1, point_2, &a_1, &a_2);
    let response = &(dlog * &challenge) + randomness;

    DleqProof(challenge, response)
}

/// Verify a DLEQ proof.
#[must_use]
pub fn verify_dleq_proof(
    proof: &DleqProof,
    base_1: &GroupElement,
    base_2: &GroupElement,
    point_1: &GroupElement,
    point_2: &GroupElement,
) -> bool {
    let a_1 = &(base_1 * &proof.1) - &(point_1 * &proof.0);
    let a_2 = &(base_2 * &proof.1) - &(point_2 * &proof.0);

    let challenge = calculate_challenge(base_1, base_2, point_1, point_2, &a_1, &a_2);
    challenge == proof.0
}

/// Calculates the challenge value.
/// Its a hash value represented as `Scalar` of all provided elements.
fn calculate_challenge(
    base_1: &GroupElement,
    base_2: &GroupElement,
    point_1: &GroupElement,
    point_2: &GroupElement,
    a_1: &GroupElement,
    a_2: &GroupElement,
) -> Scalar {
    let blake2b_hasher = Blake2b512Hasher::new()
        .chain_update(base_1.to_bytes())
        .chain_update(base_2.to_bytes())
        .chain_update(point_1.to_bytes())
        .chain_update(point_2.to_bytes())
        .chain_update(a_1.to_bytes())
        .chain_update(a_2.to_bytes());

    Scalar::from_hash(blake2b_hasher)
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    use test_strategy::proptest;

    use super::*;

    #[proptest(cases = 10)]
    fn zk_dleq_test(
        e1: Scalar,
        e2: Scalar,
        dlog1: Scalar,
        dlog2: Scalar,
        randomness: Scalar,
    ) {
        let base_1 = GroupElement::GENERATOR.mul(&e1);
        let base_2 = GroupElement::GENERATOR.mul(&e2);

        let point_1 = base_1.mul(&dlog1);
        let point_2 = base_2.mul(&dlog1);

        let proof = generate_dleq_proof(&base_1, &base_2, &point_1, &point_2, &dlog1, &randomness);
        assert!(verify_dleq_proof(
            &proof, &base_1, &base_2, &point_1, &point_2
        ));

        // use different discrete logarithm for both points
        let point_1 = base_1.mul(&dlog2);
        let point_2 = base_2.mul(&dlog2);

        let proof = generate_dleq_proof(&base_1, &base_2, &point_1, &point_2, &dlog1, &randomness);
        assert!(!verify_dleq_proof(
            &proof, &base_1, &base_2, &point_1, &point_2
        ));

        // use different discrete logarithm across points
        let point_1 = base_1.mul(&dlog1);
        let point_2 = base_2.mul(&dlog2);

        let proof = generate_dleq_proof(&base_1, &base_2, &point_1, &point_2, &dlog1, &randomness);
        assert!(!verify_dleq_proof(
            &proof, &base_1, &base_2, &point_1, &point_2
        ));
        let proof = generate_dleq_proof(&base_1, &base_2, &point_1, &point_2, &dlog2, &randomness);
        assert!(!verify_dleq_proof(
            &proof, &base_1, &base_2, &point_1, &point_2
        ));
    }
}
