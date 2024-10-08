//! Randomness and announcements structs for the ZK unit vector algorithm

#![allow(clippy::missing_docs_in_private_items)]

use std::ops::Mul;

use rand_core::CryptoRngCore;

use crate::crypto::group::{GroupElement, Scalar};

/// Randomness generated in the proof, used for the hiding property.
#[derive(Debug)]
pub struct BlindingRandomness {
    pub(crate) alpha: Scalar,
    pub(crate) betta: Scalar,
    pub(crate) gamma: Scalar,
    pub(crate) delta: Scalar,
}

impl BlindingRandomness {
    pub(crate) fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self {
            alpha: Scalar::random(rng),
            betta: Scalar::random(rng),
            gamma: Scalar::random(rng),
            delta: Scalar::random(rng),
        }
    }
}

/// First announcement, formed by I, B, A group elements. These group elements
/// are the commitments of the binary representation of the unit vector index.
pub struct Announcement {
    pub(crate) i: GroupElement,
    pub(crate) b: GroupElement,
    pub(crate) a: GroupElement,
}

impl Announcement {
    pub(crate) fn new(
        i_bit: bool, rand: &BlindingRandomness, commitment_key: &GroupElement,
    ) -> Self {
        let i = if i_bit {
            &GroupElement::GENERATOR + &commitment_key.mul(&rand.alpha)
        } else {
            commitment_key.mul(&rand.alpha)
        };
        let b = &GroupElement::GENERATOR.mul(&rand.betta) + &commitment_key.mul(&rand.gamma);
        let a = if i_bit {
            &GroupElement::GENERATOR.mul(&rand.betta) + &commitment_key.mul(&rand.delta)
        } else {
            commitment_key.mul(&rand.delta)
        };
        Self { i, b, a }
    }
}

/// Response encoding the bits of the private vector, and the randomness of
/// `BlindingRandomness`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseRandomness {
    pub(crate) z: Scalar,
    pub(crate) w: Scalar,
    pub(crate) v: Scalar,
}

impl ResponseRandomness {
    pub(crate) fn new(i_bit: bool, rand: &BlindingRandomness, com_2: &Scalar) -> Self {
        let z = if i_bit {
            com_2 + &rand.betta
        } else {
            rand.betta.clone()
        };
        let w = &(&rand.alpha * com_2) + &rand.gamma;
        let v = &(&rand.alpha * &(com_2 - &z)) + &rand.delta;
        Self { z, w, v }
    }
}

#[cfg(test)]
mod tests {
    use proptest::{
        arbitrary::any,
        prelude::{Arbitrary, BoxedStrategy, Strategy},
    };

    use super::*;

    impl Arbitrary for BlindingRandomness {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(Scalar, Scalar, Scalar, Scalar)>()
                .prop_map(|(alpha, betta, gamma, delta)| {
                    BlindingRandomness {
                        alpha,
                        betta,
                        gamma,
                        delta,
                    }
                })
                .boxed()
        }
    }
}
