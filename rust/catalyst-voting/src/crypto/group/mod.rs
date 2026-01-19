//! Group definitions used in voting protocol.
//! For more information, see: <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#a-group-definition>

mod ristretto255;

pub use ristretto255::{GroupElement, Scalar};
