//! Group definitions used in voting protocol.
//! For more information, see: <https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#a-group-definition>

mod ristretto255;
// Probably it will be used at someday, but for now it is excluded
// mod utils;

#[allow(clippy::module_name_repetitions)]
pub use ristretto255::{GroupElement, Scalar};
