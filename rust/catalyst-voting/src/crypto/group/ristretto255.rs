//! ristretto255 group implementation.

use std::ops::{Add, Mul};

use curve25519_dalek::{
    constants::{RISTRETTO_BASEPOINT_POINT, RISTRETTO_BASEPOINT_TABLE},
    ristretto::RistrettoPoint as Point,
    scalar::Scalar as IScalar,
};
use rand_core::CryptoRngCore;

/// Ristretto group scalar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(IScalar);

/// Ristretto group element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(Point);

impl GroupElement {
    /// ristretto255 group generator.
    pub const GENERATOR: GroupElement = GroupElement(RISTRETTO_BASEPOINT_POINT);
}

impl Scalar {
    /// Generate a random scalar value from the random number generator.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        let mut scalar_bytes = [0u8; 64];
        rng.fill_bytes(&mut scalar_bytes);
        Scalar(IScalar::from_bytes_mod_order_wide(&scalar_bytes))
    }
}

impl Mul<&GroupElement> for &Scalar {
    type Output = GroupElement;

    fn mul(self, other: &GroupElement) -> GroupElement {
        other * self
    }
}

impl Mul<&Scalar> for &GroupElement {
    type Output = GroupElement;

    fn mul(self, other: &Scalar) -> GroupElement {
        if self.0 == RISTRETTO_BASEPOINT_POINT {
            GroupElement(RISTRETTO_BASEPOINT_TABLE * &other.0)
        } else {
            GroupElement(other.0 * self.0)
        }
    }
}

impl Add<&GroupElement> for &GroupElement {
    type Output = GroupElement;

    fn add(self, other: &GroupElement) -> GroupElement {
        GroupElement(self.0 + other.0)
    }
}
