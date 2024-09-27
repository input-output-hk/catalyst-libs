//! Module containing all primitives related to the voter.

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

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

/// Incorrect voting choice error
#[derive(thiserror::Error, Debug)]
#[error(
    "Invalid voting choice, the value of choice: {0}, should be less than the number of voting options: {1}."
)]
pub struct IncorrectChoiceError(usize, usize);

/// Incorrect randomness length error
#[derive(thiserror::Error, Debug)]
#[error(
    "Invalid randomness length, the length of randomness: {0}, should be equal to the number of voting options: {1}."
)]
pub struct IncorrectRandomnessLengthError(usize, usize);

impl Vote {
    /// Generate a vote.
    /// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#voting-choice)
    ///
    /// # Errors
    ///   - `IncorrectChoiceError`
    pub(crate) fn new(choice: usize, voting_options: usize) -> Result<Vote, IncorrectChoiceError> {
        if choice >= voting_options {
            return Err(IncorrectChoiceError(choice, voting_options));
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

    /// Create a new encrypted vote from the given vote and public key.
    /// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#vote-encryption)
    pub(crate) fn encrypt(
        &self, public_key: &PublicKey, randomness: &EncryptionRandomness,
    ) -> Result<EncryptedVote, IncorrectRandomnessLengthError> {
        if self.voting_options != randomness.0.len() {
            return Err(IncorrectRandomnessLengthError(
                randomness.0.len(),
                self.voting_options,
            ));
        }

        let unit_vector = self.to_unit_vector();
        let ciphers = unit_vector
            .par_iter()
            .zip(randomness.0.par_iter())
            .map(|(m, r)| encrypt(m, public_key, r))
            .collect();

        Ok(EncryptedVote(ciphers))
    }
}

#[cfg(test)]
mod tests {
    use proptest::sample::size_range;
    use test_strategy::proptest;

    use super::*;
    use crate::crypto::elgamal::SecretKey;

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

    #[proptest]
    fn encrypt_test(
        secret_key: SecretKey, #[strategy(1..10usize)] voting_options: usize,
        #[any(size_range(#voting_options).lift())] randomness: Vec<Scalar>,
    ) {
        let public_key = secret_key.public_key();
        let vote = Vote::new(0, voting_options).unwrap();

        let encrypted = vote
            .encrypt(&public_key, &EncryptionRandomness(randomness))
            .unwrap();
        assert_eq!(encrypted.0.len(), vote.voting_options);
    }
}
