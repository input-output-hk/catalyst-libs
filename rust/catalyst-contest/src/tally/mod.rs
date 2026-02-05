//! Contest tally functionality with all necessary types

pub mod provider;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use anyhow::Context;
use catalyst_signed_doc::{DocumentRef, catalyst_id::CatalystId};
use catalyst_voting::vote_protocol::tally::{
    DecryptionTallySetup, decrypt_tally,
    proof::{TallyProof, generate_tally_proof_with_default_rng},
};

use crate::{
    contest_ballot::{ContestBallot, payload::Choices},
    contest_parameters::{ContestParameters, VotingOptions},
    tally::provider::TallyProvider,
    vote_protocol::{
        committee::ElectionSecretKey,
        tally::{self, EncryptedTally},
    },
};

/// Contest Tally Result type
#[derive(Debug, Clone)]
pub struct ContestResult {
    /// Contest choices, defined by the 'Contest Parameters' document
    pub options: VotingOptions,

    /// Final tally calculated per each proposal, which was assigned to the corresponding
    /// 'Contest Parameters' document
    pub tally_per_proposals: HashMap<DocumentRef, Vec<TallyPerOption>>,

    /// List of all participated voters with their voting power
    pub participants: HashMap<CatalystId, u64>,
}

/// Encrypted Tally per voting option
#[derive(Debug, Clone)]
pub struct TallyPerOption {
    /// Total sum over all clear votes
    pub clear_tally: u64,
    /// Encrypted tally (homomorphic total sum over all encrypted votes)
    pub encrypted_tally: EncryptedTally,
    /// Decrypted tally
    pub decrypted_tally: Option<DecryptedTally>,
    /// Contest voting option
    pub option: String,
}

/// Decrypted tally object with the tally result itself and its proof to the corresponding
/// `EncryptedTally`
#[derive(Debug, Clone)]
pub struct DecryptedTally {
    /// Total sum over all encrypted votes
    pub tally: u64,
    /// Encrypted tally proof
    pub proof: TallyProof,
}

impl TallyPerOption {
    /// Returns a sum of `clear_tally` and `decrypted_tally` if `decrypted_tally` if
    /// present
    #[must_use]
    pub fn total_tally(&self) -> Option<u64> {
        self.decrypted_tally
            .as_ref()
            .map(|v| self.clear_tally.saturating_add(v.tally))
    }
}

/// Contest tally procedure based on the provided 'Contest Parameters' document.
/// Collects all necessary `ContestBallot`, `Proposal`, which are associate with the
/// provided `ContestParameters`.
///
/// # Errors
///  - `provider` returns error
pub fn contest_tally(
    contest_parameters: &ContestParameters,
    election_secret_key: Option<&ElectionSecretKey>,
    provider: &dyn TallyProvider,
) -> anyhow::Result<ContestResult> {
    if let Some(election_secret_key) = election_secret_key {
        anyhow::ensure!(
            contest_parameters.election_public_key() == &election_secret_key.public_key(),
            "`election_secret_key` must align with `election_public_key` from the `contest_parameters`"
        );
    }

    let ballots = contest_parameters.get_associated_ballots(provider)?;
    let ballots = ballots
        .iter()
        .map(|d| ContestBallot::new(d, provider))
        .map(|d| {
            d.and_then(|d| {
                if d.report().is_problematic() {
                    anyhow::bail!(
                        "'Contest Ballot' document ({}) is problematic: {:?}",
                        d.doc_ref(),
                        d.report()
                    )
                }
                Ok(d)
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut ballots_with_voting_power = Vec::new();
    let mut total_voting_power: u64 = 0;
    for b in ballots {
        let voting_power = provider.try_get_voting_power(b.voter())?;
        ballots_with_voting_power.push((b, voting_power));
        total_voting_power = total_voting_power
            .checked_add(voting_power)
            .context("Total voting power overflow")?;
    }

    let decryption_credentials = election_secret_key
        .map(|key| {
            let decryption_tally_setup = DecryptionTallySetup::new(total_voting_power)?;
            anyhow::Ok((decryption_tally_setup, key.clone()))
        })
        .transpose()?;

    let proposals = contest_parameters.get_associated_proposals(provider)?;
    let tally_per_proposals = proposals
        .iter()
        .map(|p| {
            let p_ref = p.doc_ref()?;
            let tally_res = tally_per_proposal(
                &p_ref,
                &ballots_with_voting_power,
                contest_parameters.options(),
                decryption_credentials.as_ref(),
            )?;


            if decryption_credentials.is_some() {
                let total_tally_sum = tally_res.iter().map(TallyPerOption::total_tally).try_fold(0_u64, |sum, total_tally| {

                    anyhow::Ok(sum.checked_add(total_tally.context("total tally over encrypted and decrypted one must exist, because `decryption_credentials` was provided")?).context("total tally sum per proposal overflow")?)
                 })?;
                 anyhow::ensure!(
                total_tally_sum == total_voting_power,
                "The final total tally for the proposal '{total_tally_sum}' must be aligned with the total voting power '{total_voting_power}'" );
            }

            anyhow::Ok((p_ref, tally_res))
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(ContestResult {
        options: contest_parameters.options().clone(),
        tally_per_proposals,
        participants: HashMap::new(),
    })
}

// Calculates the voting tally for a specific proposal, processing both encrypted and
// clear votes.
///
/// This function aggregates votes across all provided ballots, applying the respective
/// voting power to each choice. It performs two parallel tallies:
/// 1. **Encrypted Tally**: Aggregates ciphertexts. If necessary arguments are provided  -
///    generates a decryption proof, and decrypts the result.
/// 2. **Clear Tally**: Sums plain-text votes multiplied by voting power.
fn tally_per_proposal(
    p_ref: &DocumentRef,
    ballots_with_voting_power: &[(ContestBallot, u64)],
    options: &VotingOptions,
    decryption_credentials: Option<&(DecryptionTallySetup, ElectionSecretKey)>,
) -> anyhow::Result<Vec<TallyPerOption>> {
    let choices_with_voting_power_iter = ballots_with_voting_power.iter().map(|(b, p)| {
        let c = b.get_choices_for_proposal(p_ref).context(format!(
            "'Contest Ballot' {} must have  a choice for the 'Proposal' {p_ref}",
            b.doc_ref(),
        ));
        (c, p)
    });

    let for_encrypted_choices = tally_encrypted_choices(
        &choices_with_voting_power_iter.clone(),
        options.n_options(),
        decryption_credentials,
    )?;
    let for_clear_choices =
        tally_clear_choices(choices_with_voting_power_iter, options.n_options())?;

    for_clear_choices
        .into_iter()
        .zip(for_encrypted_choices)
        .zip(options.iter().cloned())
        .map(
            |((clear_tally, (encrypted_tally, decrypted_tally)), option)| {
                anyhow::Ok(TallyPerOption {
                    clear_tally,
                    encrypted_tally,
                    decrypted_tally,
                    option,
                })
            },
        )
        .collect()
}

/// Aggregates encrypted votes using homomorphic addition and generates decryption proofs.
fn tally_encrypted_choices<'a, I>(
    choices_with_voting_power_iter: &I,
    n_options: usize,
    decryption_credentials: Option<&(DecryptionTallySetup, ElectionSecretKey)>,
) -> anyhow::Result<Vec<(EncryptedTally, Option<DecryptedTally>)>>
where
    I: Iterator<Item = (anyhow::Result<&'a Choices>, &'a u64)> + Clone,
{
    let encrypted_choices_with_voting_power_iter =
        choices_with_voting_power_iter.clone().filter_map(|(c, p)| {
            let c = c.map(|c| c.as_encrypted_choices()).transpose()?;
            Some((c.cloned(), *p))
        });

    let (mut encrypted_choices, mut encrypted_power) = (Vec::new(), Vec::new());
    for (c, p) in encrypted_choices_with_voting_power_iter {
        encrypted_choices.push(c?);
        encrypted_power.push(p);
    }

    (0..n_options)
        .map(|i| {
            let encrypted_tally =
                tally::tally(i, encrypted_choices.as_slice(), encrypted_power.as_slice())?;

            let decrypted_tally = decryption_credentials
                .as_ref()
                .map(|(decryption_tally_setup, election_secret_key)| {
                    let proof = generate_tally_proof_with_default_rng(
                        &encrypted_tally,
                        election_secret_key,
                    );

                    let tally = decrypt_tally(
                        &encrypted_tally,
                        election_secret_key,
                        decryption_tally_setup,
                    )?;
                    anyhow::Ok(DecryptedTally { tally, proof })
                })
                .transpose()?;

            anyhow::Ok((encrypted_tally, decrypted_tally))
        })
        .collect()
}

/// Aggregates clear votes by summing (choice * power) for each option.
fn tally_clear_choices<'a, I>(
    choices_with_voting_power_iter: I,
    n_options: usize,
) -> anyhow::Result<Vec<u64>>
where
    I: Iterator<Item = (anyhow::Result<&'a Choices>, &'a u64)> + Clone,
{
    let clear_choices_with_voting_power_iter =
        choices_with_voting_power_iter.filter_map(|(c, p)| {
            let c = c.map(|c| c.as_clear_choices()).transpose()?;
            Some((c.cloned(), *p))
        });

    (0..n_options)
        .map({
            |i| {
                let clear_tally = clear_choices_with_voting_power_iter.clone().try_fold(
                    0_u64,
                    |sum, (c, p)| {
                        let c = c?;
                        let c = c.get(i).context(format!(
                            "Invalid clear vote, does not have choice at voting option {i}"
                        ))?;
                        let res = c
                            .checked_mul(p)
                            .context("Multiplying voting choice to voting power overflow")?;
                        sum.checked_add(res)
                            .context("Total clear tally result overflow")
                    },
                )?;

                anyhow::Ok(clear_tally)
            }
        })
        .collect()
}
