//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! [`A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence`](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//!
//! This implementation follows this [specification](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#d-non-interactive-zk-vote-proof)

#![allow(dead_code)]

mod challenges;
mod polynomial;
mod randomness_announcements;
mod utils;

use std::ops::Mul;

use challenges::{calculate_first_challenge_hash, calculate_second_challenge_hash};
use polynomial::{calculate_polynomial_val, generate_polynomial, Polynomial};
use rand_core::CryptoRngCore;
use randomness_announcements::{Announcement, BlindingRandomness, ResponseRandomness};
use utils::get_bit;

use crate::crypto::{
    elgamal::{encrypt, Ciphertext, PublicKey},
    group::{GroupElement, Scalar},
};

/// Unit vector proof struct
pub struct UnitVectorProof(
    Vec<Announcement>,
    Vec<Ciphertext>,
    Vec<ResponseRandomness>,
    Scalar,
);

/// Generation unit vector proof error
#[derive(thiserror::Error, Debug)]
pub enum GenerationUnitVectorProofError {
    /// Incorrect number of elements
    #[error("Invalid number of elements of `unit_vector`: {0}, `encryption_randomness`: {1}, and  `ciphertexts`: {2}. They all should be equal")]
    InvalidNumberOfElements(usize, usize, usize),
}

/// Generates a unit vector proof.
///
/// `unit_vector` must be a collection of `Scalar` where only one element is equal to
/// `Scalar::one()` and the others are equal to `Scalar::zero()`.
/// Pls make sure that you are providing a correct `unit_vector`, otherwise the proof will
/// be invalid.
pub fn generate_unit_vector_proof<R: CryptoRngCore>(
    unit_vector: &[Scalar], mut encryption_randomness: Vec<Scalar>,
    mut ciphertexts: Vec<Ciphertext>, public_key: &PublicKey, commitment_key: &PublicKey,
    rng: &mut R,
) -> UnitVectorProof {
    let i = unit_vector
        .iter()
        .position(|s| s != &Scalar::zero())
        .unwrap_or(0);

    let m = unit_vector.len();
    let n = m.next_power_of_two();
    // calculates log_2(N)
    let log_n = n.trailing_zeros();

    encryption_randomness.resize(n, Scalar::zero());
    ciphertexts.resize(n, Ciphertext::zero());

    let blinding_randomness: Vec<_> = (0..log_n)
        .map(|_| BlindingRandomness::random(rng))
        .collect();

    let announcements: Vec<_> = blinding_randomness
        .iter()
        .enumerate()
        .map(|(l, r)| {
            let i_bit = get_bit(i, l);
            Announcement::new(i_bit, r, commitment_key)
        })
        .collect();

    let ch_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &announcements);
    let ch_1 = Scalar::from_hash(ch_1_hash.clone());

    let polynomials: Vec<_> = (0..n)
        .map(|j| generate_polynomial(i, j, &blinding_randomness))
        .collect();

    let (d_l, r_l) = generate_dl_and_rl(log_n, &ch_1, public_key, &polynomials, rng);

    let ch_2_hash = calculate_second_challenge_hash(ch_1_hash, &d_l);
    let ch_2 = Scalar::from_hash(ch_2_hash);

    let response_randomness: Vec<_> = blinding_randomness
        .iter()
        .enumerate()
        .map(|(l, r)| {
            let i_bit = get_bit(i, l);
            ResponseRandomness::new(i_bit, r, &ch_2)
        })
        .collect();

    // exp_ch_2 == `ch_2^(log_2(N))`
    let exp_ch_2 = (0..log_n).fold(Scalar::one(), |exp, _| exp.mul(&ch_2));

    let (p1, _) = encryption_randomness.iter().fold(
        (Scalar::zero(), Scalar::one()),
        // exp_ch_1 = `ch_1^(j)`
        |(mut sum, mut exp_ch_1), r| {
            sum = &sum + &r.mul(&exp_ch_2).mul(&exp_ch_1);
            exp_ch_1 = exp_ch_1.mul(&ch_1);
            (sum, exp_ch_1)
        },
    );
    let (p2, _) = r_l.iter().fold(
        (Scalar::zero(), Scalar::one()),
        // exp_ch_2 = `ch_2^(l)`
        |(mut sum, mut exp_ch_2), r_l| {
            sum = &sum + &r_l.mul(&exp_ch_2);
            exp_ch_2 = exp_ch_2.mul(&ch_2);
            (sum, exp_ch_2)
        },
    );
    let response = &p1 + &p2;

    UnitVectorProof(announcements, d_l, response_randomness, response)
}

/// Generates `D_l` and `R_l` elements
#[allow(clippy::indexing_slicing)]
fn generate_dl_and_rl<R: CryptoRngCore>(
    log_n: u32, ch_1: &Scalar, public_key: &PublicKey, polynomials: &[Polynomial], rng: &mut R,
) -> (Vec<Ciphertext>, Vec<Scalar>) {
    // Generate new R_l for D_l

    let r_l: Vec<_> = (0..log_n).map(|_| Scalar::random(rng)).collect();

    let d_l: Vec<_> = r_l
        .iter()
        .enumerate()
        .map(|(l, r_l)| {
            let (sum, _) = polynomials.iter().fold(
                (Scalar::zero(), Scalar::one()),
                // exp_ch_1 = `ch_1^(j)`
                |(mut sum, mut exp_ch_1), pol| {
                    sum = &sum + &pol[l].mul(&exp_ch_1);
                    exp_ch_1 = exp_ch_1.mul(ch_1);
                    (sum, exp_ch_1)
                },
            );
            encrypt(&sum, public_key, r_l)
        })
        .collect();

    (d_l, r_l)
}

/// Verify a unit vector proof.
pub fn verify_unit_vector_proof(
    proof: &UnitVectorProof, mut ciphertexts: Vec<Ciphertext>, public_key: &PublicKey,
    commitment_key: &PublicKey,
) -> bool {
    let m = ciphertexts.len();
    let n = m.next_power_of_two();
    // calculates log_2(N)
    let log_n = n.trailing_zeros();

    ciphertexts.resize(n, Ciphertext::zero());

    let ch_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &proof.0);
    let ch_1 = Scalar::from_hash(ch_1_hash.clone());

    let ch_2_hash = calculate_second_challenge_hash(ch_1_hash, &proof.1);
    let ch_2 = Scalar::from_hash(ch_2_hash);

    if !check_1(proof, &ch_2, commitment_key) {
        return false;
    }

    let left = encrypt(&Scalar::zero(), public_key, &proof.3);

    let (right_2, _) = proof.1.iter().fold(
        (Ciphertext::zero(), Scalar::one()),
        // exp_ch_2 = `ch_2^(l)`
        |(mut sum, mut exp_ch_2), d_l| {
            sum = &sum + &d_l.mul(&exp_ch_2);
            exp_ch_2 = exp_ch_2.mul(&ch_2);
            (sum, exp_ch_2)
        },
    );

    let polynomials_ch_2: Vec<_> = (0..n)
        .map(|j| calculate_polynomial_val(j, &ch_2, &proof.2))
        .collect();

    let p_j: Vec<_> = polynomials_ch_2
        .iter()
        .map(|p_ch_2| encrypt(&p_ch_2.negate(), public_key, &Scalar::zero()))
        .collect();

    // exp_ch_2 == `ch_2^(log_2(N))`
    let exp_ch_2 = (0..log_n).fold(Scalar::one(), |exp, _| exp.mul(&ch_2));

    let (right_1, _) = p_j.iter().zip(ciphertexts.iter()).fold(
        (Ciphertext::zero(), Scalar::one()),
        // exp_ch_1 = `ch_1^(j)`
        |(mut sum, mut exp_ch_1), (p_j, c_j)| {
            sum = &sum + &(&c_j.mul(&exp_ch_2) + p_j).mul(&exp_ch_1);
            exp_ch_1 = exp_ch_1.mul(&ch_1);
            (sum, exp_ch_1)
        },
    );

    let right = &right_1 + &right_2;

    right == left
}

/// Check the first part of the proof
fn check_1(proof: &UnitVectorProof, ch_2: &Scalar, commitment_key: &PublicKey) -> bool {
    proof.0.iter().zip(proof.2.iter()).all(|(an, rand)| {
        let right = &an.i.mul(ch_2) + &an.b;
        let left = &GroupElement::GENERATOR.mul(&rand.z) + &commitment_key.mul(&rand.w);
        let eq_1 = right == left;

        let right = &an.i.mul(&(ch_2 - &rand.z)) + &an.a;
        let left = &GroupElement::GENERATOR.mul(&Scalar::zero()) + &commitment_key.mul(&rand.v);
        let eq_2 = right == left;

        eq_1 && eq_2
    })
}

#[cfg(test)]
mod tests {
    use rand_core::OsRng;

    use super::{super::elgamal::SecretKey, *};

    const VECTOR_SIZE: usize = 3;

    #[test]
    fn zk_unit_vector_test() {
        let mut rng = OsRng;

        let secret_key = SecretKey::random(&mut rng);
        let secret_commitment_key = SecretKey::random(&mut rng);
        let public_key = secret_key.public_key();
        let commitment_key = secret_commitment_key.public_key();

        let unit_vector = [Scalar::one(), Scalar::zero(), Scalar::zero()];
        let encryption_randomness = vec![
            Scalar::random(&mut rng),
            Scalar::random(&mut rng),
            Scalar::random(&mut rng),
        ];

        let ciphertexts: Vec<_> = encryption_randomness
            .iter()
            .zip(unit_vector.iter())
            .map(|(r, v)| encrypt(v, &public_key, r))
            .collect();

        let proof = generate_unit_vector_proof(
            &unit_vector,
            encryption_randomness,
            ciphertexts.clone(),
            &public_key,
            &commitment_key,
            &mut rng,
        );

        assert!(verify_unit_vector_proof(
            &proof,
            ciphertexts,
            &public_key,
            &commitment_key
        ));
    }
}
