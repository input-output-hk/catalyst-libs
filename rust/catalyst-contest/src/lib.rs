//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

mod choices;
mod contest_ballot;
mod encrypted_choices;

pub use crate::{
    choices::Choices,
    contest_ballot::ContentBallot,
    encrypted_choices::{EncryptedBlock, EncryptedChoices},
};
