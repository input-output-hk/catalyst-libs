//! Module containing all primitives related to the voter.

pub mod proof;

use std::io::Read;

use anyhow::{anyhow, ensure};
use rand_core::CryptoRngCore;

use crate::crypto::{
    elgamal::{encrypt, Ciphertext, PublicKey},
    group::Scalar,
};

/// A representation of the voter's voting choice.
/// Represented as a Unit vector which size is `voting_options`
/// and the `choice` value is the index of the unit vector component equals to `1`,
/// and other components equal to `0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vote {
    /// Voter's voting choice.
    choice: usize,
    /// Number of voting options.
    voting_options: usize,
}

/// A representation of the encrypted vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedVote(Vec<Ciphertext>);

/// A representation of the encryption randomness, used to encrypt the vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptionRandomness(Vec<Scalar>);

impl EncryptionRandomness {
    /// Randomly generate the `EncryptionRandomness`.
    fn random<R: CryptoRngCore>(rng: &mut R, voting_options: usize) -> Self {
        Self((0..voting_options).map(|_| Scalar::random(rng)).collect())
    }
}

impl EncryptedVote {
    /// Decode `EncryptedVote` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode ciphertext.
    pub fn from_bytes(mut bytes: &[u8], size: usize) -> anyhow::Result<Self> {
        let mut ciph_buf = [0u8; Ciphertext::BYTES_SIZE];

        let ciphertexts = (0..size)
            .map(|i| {
                bytes.read_exact(&mut ciph_buf)?;
                Ciphertext::from_bytes(&ciph_buf)
                    .map_err(|e| anyhow!("Cannot decode ciphertext at {i}, error: {e}"))
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(Self(ciphertexts))
    }

    /// Get a deserialized bytes size
    #[must_use]
    pub fn bytes_size(&self) -> usize {
        self.0.len() * Ciphertext::BYTES_SIZE
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.bytes_size());
        self.0
            .iter()
            .for_each(|c| res.extend_from_slice(&c.to_bytes()));
        res
    }

    /// Get the ciphertext to the corresponding `voting_option`.
    pub(crate) fn get_ciphertext_for_choice(&self, voting_option: usize) -> Option<&Ciphertext> {
        self.0.get(voting_option)
    }
}

impl Vote {
    /// Generate a vote.
    /// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voting-choice)
    ///
    /// # Errors
    ///   - Invalid voting choice, the value of `choice`, should be less than the number
    ///     of `voting_options`.
    pub fn new(choice: usize, voting_options: usize) -> anyhow::Result<Vote> {
        ensure!(choice < voting_options,"Invalid voting choice, the value of choice: {choice}, should be less than the number of voting options: {voting_options}." );

        Ok(Vote {
            choice,
            voting_options,
        })
    }

    /// Transform the vote into the unit vector.
    fn to_unit_vector(&self) -> Vec<Scalar> {
        (0..self.voting_options)
            .map(|i| {
                if i == self.choice {
                    Scalar::one()
                } else {
                    Scalar::zero()
                }
            })
            .collect()
    }
}

/// Create a new encrypted vote from the given vote and public key.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#vote-encryption)
///
/// # Errors
///   - `EncryptedVoteError`
pub fn encrypt_vote<R: CryptoRngCore>(
    vote: &Vote, public_key: &PublicKey, rng: &mut R,
) -> (EncryptedVote, EncryptionRandomness) {
    let randomness = EncryptionRandomness::random(rng, vote.voting_options);

    let unit_vector = vote.to_unit_vector();
    let ciphers = unit_vector
        .iter()
        .zip(randomness.0.iter())
        .map(|(m, r)| encrypt(m, public_key, r))
        .collect();

    (EncryptedVote(ciphers), randomness)
}

#[cfg(test)]
mod tests {
    use proptest::sample::size_range;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn encrypted_vote_to_bytes_from_bytes_test(
        #[any(size_range(0..u8::MAX as usize).lift())] ciphers: Vec<Ciphertext>,
    ) {
        let vote1 = EncryptedVote(ciphers);
        let bytes = vote1.to_bytes();
        assert_eq!(bytes.len(), vote1.bytes_size());
        let vote2 = EncryptedVote::from_bytes(&bytes, vote1.0.len()).unwrap();
        assert_eq!(vote1, vote2);
    }

    #[test]
    fn vote_test() {
        let voting_options = 3;

        let vote = Vote::new(0, voting_options).unwrap();
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::one(),
            Scalar::zero(),
            Scalar::zero()
        ]);

        let vote = Vote::new(1, voting_options).unwrap();
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::zero(),
            Scalar::one(),
            Scalar::zero()
        ]);

        let vote = Vote::new(2, voting_options).unwrap();
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::zero(),
            Scalar::zero(),
            Scalar::one()
        ]);

        assert!(Vote::new(3, voting_options).is_err());
        assert!(Vote::new(4, voting_options).is_err());
    }
}
