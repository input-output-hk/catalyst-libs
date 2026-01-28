//! Module containing all primitives related to the tally process.

pub mod proof;

use std::ops::{Add, Mul};

use anyhow::{anyhow, ensure};

use super::committee::ElectionSecretKey;
use crate::{
    crypto::{
        babystep_giantstep::BabyStepGiantStep,
        elgamal::{Ciphertext, decrypt},
        group::Scalar,
    },
    vote_protocol::voter::EncryptedVote,
};

/// An important decryption tally setup, which holds an important precomputed data needed
/// for decryption.
pub struct DecryptionTallySetup {
    /// `BabyStepGiantStep` setup
    discrete_log_setup: BabyStepGiantStep,
}

/// A representation of the encrypted tally.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct EncryptedTally(Ciphertext);

impl DecryptionTallySetup {
    /// Generate a decryption tally setup.
    /// `total_voting_power` must be a total sum of all voting powers used in the `tally`
    /// procedure.
    ///
    /// **NOTE** It is a heavy operation, so please reuse the same instance for performing
    /// `decrypt_tally` function for the same `voting_powers`.
    ///
    /// # Errors
    ///   - Total voting power must more than 0.
    pub fn new(total_voting_power: u64) -> anyhow::Result<Self> {
        let discrete_log_setup =
            BabyStepGiantStep::new(total_voting_power, None).map_err(|_| {
                anyhow!("Total voting power must more than 0, provided: {total_voting_power}")
            })?;
        Ok(Self { discrete_log_setup })
    }
}

/// Tally function.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#homomorphic-tally)
///
/// # Errors
///   - Votes and voting power length mismatch.
///   - Invalid encrypted vote at index `i`. Does not have a ciphertext for the voting
///     option `voting_option`.
pub fn tally(
    voting_option: usize,
    votes: &[EncryptedVote],
    voting_powers: &[u64],
) -> anyhow::Result<EncryptedTally> {
    ensure!(
        votes.len() == voting_powers.len(),
        "Votes and voting power length mismatch. Votes amount: {0}. \
        Voting powers amount: {1}.",
        votes.len(),
        voting_powers.len(),
    );

    let ciphertexts_per_voting_option = votes
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.get_ciphertext_for_choice(voting_option).ok_or(anyhow!(
                "Invalid encrypted vote at index {i}. \
                Does not have a ciphertext for the voting option {voting_option}."
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

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

/// Decrypts the encrypted tally result.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#tally-decryption)
///
/// # Errors
///   - Cannot decrypt tally result. Provided an invalid secret key or invalid encrypted
///     tally result.
#[allow(clippy::module_name_repetitions)]
pub fn decrypt_tally(
    tally_result: &EncryptedTally,
    secret_key: &ElectionSecretKey,
    setup: &DecryptionTallySetup,
) -> anyhow::Result<u64> {
    let ge = decrypt(&tally_result.0, &secret_key.0);

    let res = setup.discrete_log_setup.discrete_log(ge).map_err(|_| {
        anyhow!(
            "Cannot decrypt tally result. \
            Provided an invalid secret key or invalid encrypted tally result."
        )
    })?;
    Ok(res)
}
