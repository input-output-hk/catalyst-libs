//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

pub mod checkpoint;
pub mod contest_ballot;
pub mod contest_delegation;
pub mod contest_parameters;

use catalyst_signed_doc::providers::CatalystSignedDocumentProvider;
use contest_parameters::{Choices, ContestParameters};

use crate::contest_ballot::ContestBallot;

/// Contest Tally Result type
#[derive(Debug, Clone)]
pub struct TallyResult {
    /// Contest choices, defined by the 'Contest Parameters' document
    #[allow(dead_code)]
    choices: Choices,
}

/// Contest tally procedure based on the provided 'Contest Parameters' document.
/// Collects all necessary `ContestBallot`, `Proposal`, `ContestDelegation` documents
/// which are associate with the provided `ContestParameters`.
///
/// # Errors
///  - `provider` returns error
pub fn tally(
    contest_parameters: &ContestParameters,
    provider: &dyn CatalystSignedDocumentProvider,
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
    let _voters = ballots.iter().map(ContestBallot::voter);
    // Filter out all invalid 'Contest Ballot' documents

    Ok(res)
}
