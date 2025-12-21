//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

pub mod contest_delegation;

mod contest_ballot;

pub use crate::contest_ballot::{
    Choices, ContentBallotPayload, ContestBallotRule, EncryptedBlock, EncryptedChoices,
};
