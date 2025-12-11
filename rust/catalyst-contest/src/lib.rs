//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

// TODO: FIXME:
//#![allow(unused_variables)]

mod choices;
mod contest_ballot;
mod encrypted_choices;
mod row_proof;

pub use crate::{
    choices::Choices,
    contest_ballot::ContentBallot,
    encrypted_choices::{EncryptedBlock, EncryptedChoices},
    row_proof::RowProof,
};
