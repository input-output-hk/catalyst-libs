//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! [`A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence`](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//!
//! This implementation follows this [specification](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#d-non-interactive-zk-vote-proof)

#![allow(dead_code, unused_variables)]

mod polynomial;
mod randomness_announcements;

use std::ops::Mul;

use curve25519_dalek::digest::Digest;
use polynomial::generate_polynomial;
use rand_core::CryptoRngCore;
use randomness_announcements::{Announcement, BlindingRandomness, ResponseRandomness};

use super::{
    elgamal::{encrypt, Ciphertext, PublicKey},
    group::{GroupElement, Scalar},
    hash::Blake2b512Hasher,
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
    /// Empty unit vector
    #[error("Provided `unit_vector` is empty")]
    EmptyUnitVector,
}

/// Generates a unit vector proof.
///
/// `unit_vector` must be a collection of `Scalar` where only one element is equal to
/// `Scalar::one()` and the others are equal to `Scalar::zero()`.
/// Pls make sure that you are providing a correct `unit_vector`, otherwise the proof will
/// be invalid.
pub fn generate_unit_vector_proof<R: CryptoRngCore>(
    unit_vector: &[Scalar], mut encryption_randomness: Vec<Scalar>,
    mut ciphertexts: Vec<Ciphertext>, public_key: &PublicKey, commitment_key: &GroupElement,
    rng: &mut R,
) -> Result<UnitVectorProof, GenerationUnitVectorProofError> {
    if unit_vector.len() != encryption_randomness.len() || unit_vector.len() != ciphertexts.len() {
        return Err(GenerationUnitVectorProofError::InvalidNumberOfElements(
            unit_vector.len(),
            encryption_randomness.len(),
            ciphertexts.len(),
        ));
    }
    let i = unit_vector
        .iter()
        .position(|s| s != &Scalar::one())
        .ok_or(GenerationUnitVectorProofError::EmptyUnitVector)?;

    let m = unit_vector.len();
    let n = m.next_power_of_two();

    encryption_randomness.resize(n, Scalar::zero());
    ciphertexts.resize(n, Ciphertext::zero());

    // calculates log_2(N)
    let log_n = n.trailing_zeros();

    let i_bits = bin_rep(i, log_n);

    let blinding_randomness: Vec<_> = i_bits
        .iter()
        .map(|_| BlindingRandomness::random(rng))
        .collect();

    let announcements: Vec<_> = blinding_randomness
        .iter()
        .zip(i_bits.iter())
        .map(|(r, i_bit)| Announcement::new(*i_bit, r, commitment_key))
        .collect();

    let com_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &announcements);
    let com_1 = Scalar::from_hash(com_1_hash.clone());

    let polynomials: Vec<_> = (0..n)
        .map(|j| generate_polynomial(&i_bits, &blinding_randomness, j, log_n))
        .collect();

    // Generate new R_l for D_l
    let mut rs = Vec::with_capacity(log_n as usize);
    let mut ds = Vec::with_capacity(log_n as usize);

    #[allow(clippy::indexing_slicing)]
    for i in 0..log_n {
        let (sum, _) = polynomials.iter().fold(
            (Scalar::zero(), Scalar::one()),
            |(mut sum, mut exp_com_1), pol| {
                sum = &sum + &pol[(log_n - 1 - i) as usize].mul(&exp_com_1);
                exp_com_1 = exp_com_1.mul(&com_1);
                (sum, exp_com_1)
            },
        );

        let r_l = Scalar::random(rng);
        let d_l = encrypt(&sum, public_key, &r_l);

        rs.push(r_l);
        ds.push(d_l);
    }

    let com_2_hash = calculate_second_challenge_hash(com_1_hash, &ds);
    let com_2 = Scalar::from_hash(com_2_hash);

    let response_randomness: Vec<_> = blinding_randomness
        .iter()
        .zip(i_bits.iter())
        .map(|(r, i_bit)| ResponseRandomness::new(*i_bit, r, &com_2))
        .collect();

    let response = {
        let exp_com_2 = (0..=log_n).fold(Scalar::one(), |exp, _| exp.mul(&com_2));
        let (p1, _) = encryption_randomness.iter().fold(
            (Scalar::zero(), Scalar::one()),
            |(mut sum, mut exp_com_1), r| {
                sum = &sum + &(&(r * &exp_com_2) * &exp_com_1);
                exp_com_1 = exp_com_1.mul(&com_1);
                (sum, exp_com_1)
            },
        );
        let (p2, _) = rs.iter().fold(
            (Scalar::zero(), Scalar::one()),
            |(mut sum, mut exp_com_2), r| {
                sum = &sum + &(r * &exp_com_2);
                exp_com_2 = exp_com_2.mul(&com_2);
                (sum, exp_com_2)
            },
        );
        &p1 + &p2
    };

    Ok(UnitVectorProof(
        announcements,
        ds,
        response_randomness,
        response,
    ))
}

/// Generates a binary representation vector of `val` with the given `size`
fn bin_rep(val: usize, size: u32) -> Vec<bool> {
    (0..size)
        .map(|bit_index| ((val.to_le() >> bit_index) & 1) == 1)
        .collect()
}

/// Calculates the first challenge hash.
fn calculate_first_challenge_hash(
    commitment_key: &GroupElement, public_key: &PublicKey, ciphertexts: &[Ciphertext],
    announcements: &[Announcement],
) -> Blake2b512Hasher {
    let mut hash = Blake2b512Hasher::new()
        .chain_update(commitment_key.to_bytes())
        .chain_update(public_key.to_bytes());
    for c in ciphertexts {
        hash.update(c.first().to_bytes());
        hash.update(c.second().to_bytes());
    }
    for announcement in announcements {
        hash.update(announcement.i.to_bytes());
        hash.update(announcement.b.to_bytes());
        hash.update(announcement.a.to_bytes());
    }
    hash
}

/// Calculates the second challenge hash.
fn calculate_second_challenge_hash(
    mut com_1_hash: Blake2b512Hasher, ciphertexts: &[Ciphertext],
) -> Blake2b512Hasher {
    for c in ciphertexts {
        com_1_hash.update(c.first().to_bytes());
        com_1_hash.update(c.second().to_bytes());
    }
    com_1_hash
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        println!("{}", 4_u32.trailing_zeros());
    }
}
