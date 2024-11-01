//! Random number generator objects.

// cspell: words Seedable

use rand_chacha::ChaCha8Rng;
pub use rand_core;
use rand_core::{CryptoRngCore, SeedableRng};

/// Default random number generator `rand_chacha::ChaCha8Rng`.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn default_rng() -> impl CryptoRngCore {
    ChaCha8Rng::from_entropy()
}
