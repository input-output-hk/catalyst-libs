//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

pub mod checkpoint;
pub mod contest_delegation;
pub mod contest_parameters;

mod contest_ballot;

use catalyst_signed_doc::providers::CatalystSignedDocumentProvider;

pub use crate::contest_ballot::{
    Choices, ContentBallotPayload, ContestBallot, ContestBallotRule, EncryptedBlock,
    EncryptedChoices,
};
use crate::contest_parameters::ContestParameters;

/// Contest tally procedure based on the provided 'Contest Parameters' document.
/// Collects all necessary `ContestBallot`, `Proposal`, `ContestDelegation` documents
/// which are associate with the provided `ContestParameters`.
///
/// Filling the provided `contest_parameters` problem report if something goes wrong.
///
/// # Errors
///  - `provider` returns error
pub fn tally(
    contest_parameters: &ContestParameters,
    provider: &dyn CatalystSignedDocumentProvider,
) -> anyhow::Result<()> {
    let _proposals = contest_parameters.get_associated_proposals(provider)?;

    Ok(())
}
