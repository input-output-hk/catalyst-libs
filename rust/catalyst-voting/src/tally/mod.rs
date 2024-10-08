//! Module containing all primitives related to the tally process.

pub mod proof;

use std::ops::{Add, Mul};

use crate::{
    crypto::{
        babystep_giantstep::BabyStepGiantStep,
        elgamal::{decrypt, Ciphertext, SecretKey},
        group::Scalar,
    },
    voter::EncryptedVote,
};

/// An important decryption tally setup, which holds an important precomputed data needed
/// for decryption.
pub struct DecryptionTallySetup {
    /// `BabyStepGiantStep` setup
    discrete_log_setup: BabyStepGiantStep,
}

/// A representation of the encrypted tally.
#[allow(clippy::module_name_repetitions)]
pub struct EncryptedTally(Ciphertext);

/// Tally error
#[derive(thiserror::Error, Debug)]
pub enum DecryptionTallySetupError {
    /// Votes and voting power mismatch
    #[error("Total voting power must more than 0.")]
    InvalidTotalVotingPowerAmount,
}

impl DecryptionTallySetup {
    /// Generate a decryption tally setup.
    /// `total_voting_power` must be a total sum of all voting powers used in the `tally`
    /// procedure.
    ///
    /// **NOTE** It is a heavy operation, so please reuse the same instance for performing
    /// `decrypt_tally` function for the same `voting_powers`.
    ///
    /// # Errors
    ///   - `DecryptionTallySetupError`
    pub fn new(total_voting_power: u64) -> Result<Self, DecryptionTallySetupError> {
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
) -> Result<EncryptedTally, TallyError> {
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
        .iter()
        .zip(voting_powers.iter())
        .map(|(ciphertext, voting_power)| {
            let voting_power_scalar = Scalar::from(*voting_power);
            ciphertext.mul(&voting_power_scalar)
        })
        .fold(zero_ciphertext, |acc, c| acc.add(&c));

    Ok(EncryptedTally(res))
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
#[allow(clippy::module_name_repetitions)]
pub fn decrypt_tally(
    tally_result: &EncryptedTally, secret_key: &SecretKey, setup: &DecryptionTallySetup,
) -> Result<u64, DecryptTallyError> {
    let ge = decrypt(&tally_result.0, secret_key);

    let res = setup
        .discrete_log_setup
        .discrete_log(ge)
        .map_err(|_| DecryptTallyError::CannotDecryptTallyResult)?;
    Ok(res)
}
