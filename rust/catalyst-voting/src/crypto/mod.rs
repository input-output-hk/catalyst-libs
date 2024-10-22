//! Crypto primitives which are used by voting protocol.

// cspell: words Seedable

use rand_chacha::ChaCha8Rng;
use rand_core::{CryptoRngCore, SeedableRng};

pub mod babystep_giantstep;
pub mod ed25519;
pub mod elgamal;
pub mod group;
pub mod hash;
pub mod zk_dl_equality;
pub mod zk_unit_vector;

/// Default random number generator `rand_chacha::ChaCha8Rng`.
#[must_use]
pub fn default_rng() -> impl CryptoRngCore {
    ChaCha8Rng::from_entropy()
}
