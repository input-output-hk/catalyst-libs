//! Randomness and announcements structs for the ZK unit vector algorithm

#![allow(clippy::missing_docs_in_private_items, dead_code)]

use std::ops::Mul;

use rand_core::CryptoRngCore;

use crate::crypto::group::{GroupElement, Scalar};

/// Randomness generated in the proof, used for the hiding property.
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Announcement {
    pub(crate) i: GroupElement,
    pub(crate) b: GroupElement,
    pub(crate) a: GroupElement,
}

/// `EncryptedVote` decoding error
#[derive(thiserror::Error, Debug)]
pub enum AnnouncementDecodingError {
    /// Cannot decode ciphertext
    #[error("Cannot decode group element {0} field.")]
    CannotDecodeGroupElement(char),
}

impl Announcement {
    /// `Announcement` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE * 3;

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

    /// Decode `Announcement` from bytes.
    ///
    /// # Errors
    ///   - `AnnouncementDecodingError`
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> Result<Self, AnnouncementDecodingError> {
        let i = GroupElement::from_bytes(bytes[0..32].try_into().unwrap())
            .ok_or(AnnouncementDecodingError::CannotDecodeGroupElement('i'))?;
        let b = GroupElement::from_bytes(bytes[32..64].try_into().unwrap())
            .ok_or(AnnouncementDecodingError::CannotDecodeGroupElement('b'))?;
        let a = GroupElement::from_bytes(bytes[64..96].try_into().unwrap())
            .ok_or(AnnouncementDecodingError::CannotDecodeGroupElement('a'))?;
        Ok(Self { i, b, a })
    }

    /// Encode `Announcement` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; 96];
        res[0..32].copy_from_slice(&self.i.to_bytes());
        res[32..64].copy_from_slice(&self.b.to_bytes());
        res[64..96].copy_from_slice(&self.a.to_bytes());
        res
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

/// `EncryptedVote` decoding error
#[derive(thiserror::Error, Debug)]
pub enum ResponseRandomnessDecodingError {
    /// Cannot decode ciphertext
    #[error("Cannot decode scalar {0} field.")]
    CannotDecodeScalar(char),
}

impl ResponseRandomness {
    /// `ResponseRandomness` bytes size
    pub const BYTES_SIZE: usize = Scalar::BYTES_SIZE * 3;

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

    /// Decode `ResponseRandomness` from bytes.
    ///
    /// # Errors
    ///   - `ResponseRandomnessDecodingError`
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(
        bytes: &[u8; Self::BYTES_SIZE],
    ) -> Result<Self, ResponseRandomnessDecodingError> {
        let z = Scalar::from_bytes(bytes[0..32].try_into().unwrap())
            .ok_or(ResponseRandomnessDecodingError::CannotDecodeScalar('z'))?;
        let w = Scalar::from_bytes(bytes[32..64].try_into().unwrap())
            .ok_or(ResponseRandomnessDecodingError::CannotDecodeScalar('w'))?;
        let v = Scalar::from_bytes(bytes[64..96].try_into().unwrap())
            .ok_or(ResponseRandomnessDecodingError::CannotDecodeScalar('v'))?;
        Ok(Self { z, w, v })
    }

    /// Encode `ResponseRandomness` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; 96];
        res[0..32].copy_from_slice(&self.z.to_bytes());
        res[32..64].copy_from_slice(&self.w.to_bytes());
        res[64..96].copy_from_slice(&self.v.to_bytes());
        res
    }
}

#[cfg(test)]
mod tests {
    use proptest::{
        arbitrary::any,
        prelude::{Arbitrary, BoxedStrategy, Strategy},
    };
    use test_strategy::proptest;

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

    impl Arbitrary for Announcement {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(GroupElement, GroupElement, GroupElement)>()
                .prop_map(|(i, b, a)| Announcement { i, b, a })
                .boxed()
        }
    }

    impl Arbitrary for ResponseRandomness {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(Scalar, Scalar, Scalar)>()
                .prop_map(|(z, w, v)| ResponseRandomness { z, w, v })
                .boxed()
        }
    }

    #[proptest]
    fn announcement_to_bytes_from_bytes_test(a1: Announcement) {
        let bytes = a1.to_bytes();
        let a2 = Announcement::from_bytes(&bytes).unwrap();
        assert_eq!(a1, a2);
    }

    #[proptest]
    fn response_randomness_to_bytes_from_bytes_test(r1: ResponseRandomness) {
        let bytes = r1.to_bytes();
        let r2 = ResponseRandomness::from_bytes(&bytes).unwrap();
        assert_eq!(r1, r2);
    }
}
