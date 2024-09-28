//! Group definitions used in voting protocol.
//! For more information, see: <https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#a-group-definition>

mod babystep_giantstep;
mod ristretto255;

pub(crate) use babystep_giantstep::BabyStepGiantStep;
pub(crate) use ristretto255::{GroupElement, Scalar};
