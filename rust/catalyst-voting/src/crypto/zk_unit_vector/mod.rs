//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! [`A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence`](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//!
//! This implementation follows this [specification](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#d-non-interactive-zk-vote-proof)

#![allow(dead_code, unused_variables)]

mod randomness_announcements;

use curve25519_dalek::digest::Digest;
use rand_core::CryptoRngCore;
use randomness_announcements::{Announcement, Randomness};

use super::{
    elgamal::{Ciphertext, PublicKey},
    group::{GroupElement, Scalar},
    hash::Blake2b512Hasher,
};

/// Unit vector proof struct
pub struct UnitVectorProof;

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

    let i_bits: Vec<_> = (0..log_n)
        .map(|bit_index| ((i.to_le() >> bit_index) & 1) == 1)
        .collect();

    let randomness: Vec<_> = i_bits.iter().map(|_| Randomness::random(rng)).collect();

    let announcements: Vec<_> = randomness
        .iter()
        .zip(i_bits.iter())
        .map(|(r, i_bit)| Announcement::new(*i_bit, r, commitment_key))
        .collect();

    let com_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &announcements);
    let com_1 = Scalar::from_hash(com_1_hash);

    Ok(UnitVectorProof)
}

/// Calculates the first challenge value.
/// Its a hash value represented as `Scalar` of all provided elements.
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

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        println!("{}", 4_u32.trailing_zeros());
    }
}
