//! A Catalyst voting (contest) functionality.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot/

// TODO: FIXME:
#![allow(unused_variables)]

mod choices;
mod column_proof;
mod contest_ballot;
mod encrypted_choices;
mod matrix_proof;

pub use crate::{
    choices::{Choices, ElgamalRistretto255Choice, RowProof},
    column_proof::ColumnProof,
    contest_ballot::ContentBallot,
    encrypted_choices::{EncryptedBlock, EncryptedChoices},
    matrix_proof::MatrixProof,
};
