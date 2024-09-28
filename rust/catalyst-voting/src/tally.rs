//! Module containing all primitives related to the tally process.

use std::ops::{Add, Mul};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    crypto::{
        elgamal::{decrypt, Ciphertext, SecretKey},
        group::{BabyStepGiantStep, Scalar},
    },
    voter::EncryptedVote,
};

/// An important decription tally setup, which holds an important precomputed data needed
/// for decryption.
pub struct DecriptionTallySetup {
    /// `BabyStepGiantStep` setup
    discrete_log_setup: BabyStepGiantStep,
}

/// A representation of the encrypted tally result.
pub struct EncryptedTallyResult(Ciphertext);

/// Tally error
#[derive(thiserror::Error, Debug)]
pub enum DecryptionTallySetupError {
    /// Votes and voting power mismatch
    #[error("Total voting power must more than 0.")]
    InvalidTotalVotingPowerAmount,
}

impl DecriptionTallySetup {
    /// Generate a decryption tally setup.
    ///
    /// # Errors
    ///   - `DecryptionTallySetupError`
    pub fn new(voting_powers: &[u64]) -> Result<Self, DecryptionTallySetupError> {
        let total_voting_power = voting_powers.par_iter().sum();
        let discrete_log_setup = BabyStepGiantStep::new(total_voting_power, None)
            .map_err(|_| DecryptionTallySetupError::InvalidTotalVotingPowerAmount)?;
        Ok(Self { discrete_log_setup })
    }
}

/// Tally error
#[derive(thiserror::Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum TallyError {
    /// Votes and voting power mismatch
    #[error("Votes and voting power mismatch. Votes amount: {0}. Voting powers amount: {1}.")]
    VotingPowerAndVotesMismatch(usize, usize),
    /// Invalid encrypted vote
    #[error("Invalid encrypted vote at index {0}. Does not have a ciphertext for the voting option {1}.")]
    InvalidEncryptedVote(usize, usize),
}

/// Tally function.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#homomorphic-tally)
///
/// # Errors
///   - `TallyError`
pub fn tally(
    voting_option: usize, votes: &[EncryptedVote], voting_powers: &[u64],
) -> Result<EncryptedTallyResult, TallyError> {
    if votes.len() != voting_powers.len() {
        return Err(TallyError::VotingPowerAndVotesMismatch(
            votes.len(),
            voting_powers.len(),
        ));
    }

    let mut ciphertexts_per_voting_option = Vec::new();
    for (i, vote) in votes.iter().enumerate() {
        let ciphertext = vote
            .get_ciphertext_for_choice(voting_option)
            .ok_or(TallyError::InvalidEncryptedVote(i, voting_option))?;
        ciphertexts_per_voting_option.push(ciphertext);
    }

    let zero_ciphertext = Ciphertext::zero();

    let res = ciphertexts_per_voting_option
        .par_iter()
        .zip(voting_powers.par_iter())
        .map(|(ciphertext, voting_power)| {
            let voting_power_scalar = Scalar::from(*voting_power);
            ciphertext.mul(&voting_power_scalar)
        })
        .reduce(
            || zero_ciphertext.clone(),
            |res, ciphertext| res.add(&ciphertext),
        );

    Ok(EncryptedTallyResult(res))
}

/// Tally error
#[derive(thiserror::Error, Debug)]
pub enum DecryptTallyError {
    /// Cannot decrypt tally result
    #[error(
        "Cannot decrypt tally result. Provided an invalid secret key or invalid encrypted tally result."
    )]
    CannotDecryptTallyResult,
}

/// Decrypts the encrypted tally result.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#tally-decryption)
///
/// # Errors
///   - `DecryptTallyError`
pub fn decrypt_tally_result(
    tally_result: &EncryptedTallyResult, secret_key: &SecretKey, setup: &DecriptionTallySetup,
) -> Result<u64, DecryptTallyError> {
    let ge = decrypt(&tally_result.0, secret_key);

    let res = setup
        .discrete_log_setup
        .discrete_log(ge)
        .map_err(|_| DecryptTallyError::CannotDecryptTallyResult)?;
    Ok(res)
}
