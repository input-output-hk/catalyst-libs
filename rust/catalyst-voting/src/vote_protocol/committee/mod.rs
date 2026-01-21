//! Module containing all primitives related to the committee.

mod decoding;

use crate::crypto::{
    elgamal::generate_public_key,
    group::{GroupElement, Scalar},
    rng::{default_rng, rand_core::CryptoRngCore},
};

/// Election secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionSecretKey(pub(crate) Scalar);

impl ElectionSecretKey {
    /// Randomly generate the `ElectionSecretKey`.
    #[must_use]
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self(Scalar::random(rng))
    }

    /// Randomly generate the `ElectionSecretKey` with the `crypto::default_rng`.
    #[must_use]
    pub fn random_with_default_rng() -> Self {
        Self::random(&mut default_rng())
    }

    /// Generate a corresponding `PublicKey`.
    #[must_use]
    pub fn public_key(&self) -> ElectionPublicKey {
        ElectionPublicKey(generate_public_key(&self.0))
    }
}

/// Election public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionPublicKey(pub(crate) GroupElement);

impl From<GroupElement> for ElectionPublicKey {
    fn from(value: GroupElement) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod arbitrary_impl {
    use proptest::prelude::{Arbitrary, BoxedStrategy, Strategy, any};

    use super::{ElectionSecretKey, Scalar};

    impl Arbitrary for ElectionSecretKey {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<Scalar>().prop_map(Self).boxed()
        }
    }
}
