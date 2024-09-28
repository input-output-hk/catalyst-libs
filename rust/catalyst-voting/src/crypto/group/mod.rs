//! Group definitions used in voting protocol.
//! For more information, see: <https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#a-group-definition>

mod babystep;
mod ristretto255;

#[allow(clippy::module_name_repetitions)]
pub use ristretto255::{GroupElement, Scalar};
