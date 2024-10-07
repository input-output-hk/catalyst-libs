//! Module containing all primitives related to the voter.

use rand_core::CryptoRngCore;

use crate::crypto::{
    elgamal::{encrypt, Ciphertext, PublicKey},
    group::Scalar,
};

/// A representation of the voter's voting choice.
/// Represented as a Unit vector which size is `voting_options`
/// and the `choice` value is the index of the unit vector component equals to `1`,
/// and other components equal to `0`.
pub struct Vote {
    /// Voter's voting choice.
    choice: usize,
    /// Number of voting options.
    voting_options: usize,
}

/// A representation of the encrypted vote.
pub struct EncryptedVote(Vec<Ciphertext>);

/// A representation of the encryption randomness, used to encrypt the vote.
pub struct EncryptionRandomness(Vec<Scalar>);

impl EncryptionRandomness {
    /// Randomly generate the `EncryptionRandomness`.
    fn generate<R: CryptoRngCore>(rng: &mut R, voting_options: usize) -> Self {
        Self((0..voting_options).map(|_| Scalar::random(rng)).collect())
    }
}

impl EncryptedVote {
    /// Get the ciphertext to the corresponding `voting_option`.
    pub(crate) fn get_ciphertext_for_choice(&self, voting_option: usize) -> Option<&Ciphertext> {
        self.0.get(voting_option)
    }
}

/// Encrypted vote error
#[derive(thiserror::Error, Debug)]
pub enum VoteError {
    /// Incorrect voting choice
    #[error(
        "Invalid voting choice, the value of choice: {0}, should be less than the number of voting options: {1}."
    )]
    IncorrectChoiceError(usize, usize),
}

impl Vote {
    /// Generate a vote.
    /// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voting-choice)
    ///
    /// # Errors
    ///   - `VoteError`
    pub fn new(choice: usize, voting_options: usize) -> Result<Vote, VoteError> {
        if choice >= voting_options {
            return Err(VoteError::IncorrectChoiceError(choice, voting_options));
        }

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
    let randomness = EncryptionRandomness::generate(rng, vote.voting_options);

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
    use super::*;

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
