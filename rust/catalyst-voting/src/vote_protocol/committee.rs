//! Module containing all primitives related to the committee.

use rand_core::CryptoRngCore;

use crate::crypto::{
    default_rng,
    elgamal::generate_public_key,
    group::{GroupElement, Scalar},
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

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};

    use super::*;

    impl Arbitrary for ElectionSecretKey {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<Scalar>().prop_map(Self).boxed()
        }
    }
}
