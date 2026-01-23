//! Module containing all primitives related to the voter.

mod decoding;
pub mod proof;

use anyhow::{anyhow, bail, ensure};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use super::committee::{ElectionPublicKey, ElectionSecretKey};
use crate::crypto::{
    babystep_giantstep::BabyStepGiantStep,
    elgamal::{Ciphertext, decrypt, encrypt},
    group::Scalar,
    rng::{default_rng, rand_core::CryptoRngCore},
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncryptedVote(Vec<Ciphertext>);

/// A representation of the encryption randomness, used to encrypt the vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptionRandomness(Vec<Scalar>);

impl EncryptionRandomness {
    /// Randomly generate the `EncryptionRandomness`.
    fn random<R: CryptoRngCore>(
        rng: &mut R,
        voting_options: usize,
    ) -> Self {
        Self((0..voting_options).map(|_| Scalar::random(rng)).collect())
    }
}

impl EncryptedVote {
    /// Returns the number of voting options
    pub fn n_options(&self) -> usize {
        self.0.len()
    }

    /// Get the ciphertext to the corresponding `voting_option`.
    pub(crate) fn get_ciphertext_for_choice(
        &self,
        voting_option: usize,
    ) -> Option<&Ciphertext> {
        self.0.get(voting_option)
    }
}

impl Vote {
    /// Generate a vote.
    /// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#voting-choice)
    ///
    /// # Errors
    ///   - Invalid voting choice, the value of `choice`, should be less than the number
    ///     of `voting_options`.
    pub fn new(
        choice: usize,
        voting_options: usize,
    ) -> anyhow::Result<Vote> {
        ensure!(
            choice < voting_options,
            "Invalid voting choice, the value of choice: {choice}, should be less than the number of voting options: {voting_options}."
        );

        Ok(Vote {
            choice,
            voting_options,
        })
    }

    /// Get the voter's choice.
    #[must_use]
    pub fn choice(&self) -> usize {
        self.choice
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

/// Create a new encrypted vote from the given vote and public key with with the
/// `crypto::default_rng`. More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#vote-encryption)
///
/// # Errors
///   - `EncryptedVoteError`
#[must_use]
pub fn encrypt_vote<R: CryptoRngCore>(
    vote: &Vote,
    public_key: &ElectionPublicKey,
    rng: &mut R,
) -> (EncryptedVote, EncryptionRandomness) {
    let randomness = EncryptionRandomness::random(rng, vote.voting_options);

    let unit_vector = vote.to_unit_vector();
    let ciphers = unit_vector
        .par_iter()
        .zip(randomness.0.par_iter())
        .map(|(m, r)| encrypt(m, &public_key.0, r))
        .collect();

    (EncryptedVote(ciphers), randomness)
}

/// Create a new encrypted vote from the given vote and public key.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#vote-encryption)
#[must_use]
pub fn encrypt_vote_with_default_rng(
    vote: &Vote,
    public_key: &ElectionPublicKey,
) -> (EncryptedVote, EncryptionRandomness) {
    encrypt_vote(vote, public_key, &mut default_rng())
}

/// Decrypt the encrypted vote.
/// **NOTE** make sure tha the provided `vote` is a valid one, by executing the
/// `verify_voter_proof` on the underlying voter proof.
/// If not valid encrypted vote is provided, unexpected results may occur.
///
/// # Errors
///   - Invalid encrypted vote, not a valid unit vector.
pub fn decrypt_vote(
    vote: &EncryptedVote,
    secret_key: &ElectionSecretKey,
) -> anyhow::Result<Vote> {
    // Assuming that the provided encrypted vote is a correctly encoded unit vector,
    // the maximum log value is `1`.
    let setup = BabyStepGiantStep::new(1, None)?;

    for (i, encrypted_choice_per_option) in vote.0.iter().enumerate() {
        let decrypted_choice_per_option = decrypt(encrypted_choice_per_option, &secret_key.0);
        let choice_per_option = setup
            .discrete_log(decrypted_choice_per_option)
            .map_err(|_| anyhow!("Invalid encrypted vote, not a valid unit vector."))?;
        if choice_per_option == 1 {
            return Ok(Vote {
                choice: i,
                voting_options: vote.0.len(),
            });
        }
    }
    bail!("Invalid encrypted vote, not a valid unit vector.")
}

impl From<Vec<Ciphertext>> for EncryptedVote {
    fn from(value: Vec<Ciphertext>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod arbitrary_impl {
    use proptest::{
        prelude::{Arbitrary, BoxedStrategy, Strategy, any_with},
        sample::size_range,
    };

    use super::{Ciphertext, EncryptedVote};

    impl Arbitrary for EncryptedVote {
        type Parameters = usize;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(size: Self::Parameters) -> Self::Strategy {
            any_with::<Vec<Ciphertext>>((size_range(size), ()))
                .prop_map(Self)
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_test() {
        let voting_options = 3;

        let vote = Vote::new(0, voting_options).unwrap();
        assert_eq!(vote.choice(), 0);
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::one(),
            Scalar::zero(),
            Scalar::zero()
        ]);

        let vote = Vote::new(1, voting_options).unwrap();
        assert_eq!(vote.choice(), 1);
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::zero(),
            Scalar::one(),
            Scalar::zero()
        ]);

        let vote = Vote::new(2, voting_options).unwrap();
        assert_eq!(vote.choice(), 2);
        assert_eq!(vote.to_unit_vector(), vec![
            Scalar::zero(),
            Scalar::zero(),
            Scalar::one()
        ]);

        assert!(Vote::new(3, voting_options).is_err());
        assert!(Vote::new(4, voting_options).is_err());
    }
}
