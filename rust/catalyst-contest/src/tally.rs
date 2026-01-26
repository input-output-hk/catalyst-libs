//! Contest tally functionality with all necessary types

use std::collections::HashMap;

use catalyst_signed_doc::{catalyst_id::CatalystId, providers::CatalystSignedDocumentProvider};
use catalyst_voting::vote_protocol::tally::DecryptionTallySetup;

use crate::{
    contest_ballot::ContestBallot,
    contest_parameters::{ContestParameters, VotingOptions},
};

/// Contest Tally Result type
#[derive(Debug, Clone)]
pub struct TallyResult {
    /// Contest choices, defined by the 'Contest Parameters' document
    #[allow(dead_code)]
    choices: VotingOptions,
}

/// Voter's voting power provider
pub trait VotingPowerProvider: CatalystSignedDocumentProvider {
    /// Try to get a voting power value by the provided user's `CatalystId`.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    fn try_get_voting_power(
        &self,
        kid: &CatalystId,
    ) -> anyhow::Result<u64>;
}

/// Contest tally procedure based on the provided 'Contest Parameters' document.
/// Collects all necessary `ContestBallot`, `Proposal`, `ContestDelegation` documents
/// which are associate with the provided `ContestParameters`.
///
/// # Errors
///  - `provider` returns error
pub fn tally(
    contest_parameters: &ContestParameters,
    provider: &dyn VotingPowerProvider,
) -> anyhow::Result<TallyResult> {
    let res = TallyResult {
        choices: contest_parameters.choices().clone(),
    };

    let _proposals = contest_parameters.get_associated_proposals(provider)?;

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

    let voters = ballots
        .iter()
        .map(|d| Ok((d.voter(), provider.try_get_voting_power(d.voter())?)))
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    let total_voting_power = voters.values().sum::<u64>();
    let _decryption_tally_setup = DecryptionTallySetup::new(total_voting_power)?;

    Ok(res)
}

// fn tally_per_proposal(
//     proposal_ref: DocumentRef,
//     ballots: &[ContestBallot],
// ) -> anyhow::Result<()> {
//     Ok(())
// }
