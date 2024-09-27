//! ristretto255 group implementation.

// cspell: words BASEPOINT

use std::ops::{Add, Mul, Sub};

use curve25519_dalek::{
    constants::{RISTRETTO_BASEPOINT_POINT, RISTRETTO_BASEPOINT_TABLE},
    ristretto::RistrettoPoint as Point,
    scalar::Scalar as IScalar,
    traits::Identity,
};
use rand_core::CryptoRngCore;

/// Ristretto group scalar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(IScalar);

/// Ristretto group element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(Point);

impl From<u64> for Scalar {
    fn from(value: u64) -> Self {
        Scalar(IScalar::from(value))
    }
}

impl Scalar {
    /// Generate a random scalar value from the random number generator.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        let mut scalar_bytes = [0u8; 64];
        rng.fill_bytes(&mut scalar_bytes);
        Scalar(IScalar::from_bytes_mod_order_wide(&scalar_bytes))
    }

    /// additive identity
    pub fn zero() -> Self {
        Scalar(IScalar::ZERO)
    }

    /// multiplicative identity
    pub fn one() -> Self {
        Scalar(IScalar::ONE)
    }

    /// negative value
    pub fn negate(&self) -> Self {
        Scalar(-self.0)
    }

    /// multiplicative inverse value, like `1 / Scalar`.
    pub fn inverse(&self) -> Scalar {
        Scalar(self.0.invert())
    }
}

impl GroupElement {
    /// ristretto255 group generator.
    pub const GENERATOR: GroupElement = GroupElement(RISTRETTO_BASEPOINT_POINT);

    /// Generate a zero group element.
    pub fn zero() -> Self {
        GroupElement(Point::identity())
    }
}

// `std::ops` traits implementations

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

impl Mul<&Scalar> for &Scalar {
    type Output = Scalar;

    fn mul(self, other: &Scalar) -> Scalar {
        Scalar(self.0 * other.0)
    }
}

impl Add<&GroupElement> for &GroupElement {
    type Output = GroupElement;

    fn add(self, other: &GroupElement) -> GroupElement {
        GroupElement(self.0 + other.0)
    }
}

impl Add<&Scalar> for &Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Scalar(self.0 + other.0)
    }
}

impl Sub<&Scalar> for &Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Scalar(self.0 - other.0)
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

    impl Arbitrary for Scalar {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<u64>().prop_map(Scalar::from).boxed()
        }
    }

    #[proptest]
    fn scalar_arithmetic_tests(e1: Scalar, e2: Scalar, e3: Scalar) {
        assert_eq!(&(&e1 + &e2) + &e3, &e1 + &(&e2 + &e3));
        assert_eq!(&e1 + &e2, &e2 + &e1);
        assert_eq!(&e1 + &Scalar::zero(), e1.clone());
        assert_eq!(&e1 * &Scalar::one(), e1.clone());
        assert_eq!(&e1 * &e1.inverse(), Scalar::one());
        assert_eq!(&e1 + &e1.negate(), Scalar::zero());
        assert_eq!(&(&e1 - &e2) + &e2, e1.clone());
        assert_eq!(&(&e1 + &e2) * &e3, &(&e1 * &e3) + &(&e2 * &e3));
    }

    #[proptest]
    fn group_element_arithmetic_tests(e1: Scalar, e2: Scalar) {
        let ge = GroupElement::GENERATOR.mul(&e1);
        assert_eq!(&GroupElement::zero() + &ge, ge);

        let ge1 = GroupElement::GENERATOR.mul(&e1);
        let ge2 = GroupElement::GENERATOR.mul(&e2);
        let ge3 = GroupElement::GENERATOR.mul(&(&e1 + &e2));

        assert_eq!(&ge1 + &ge2, ge3);

        let ge = GroupElement::GENERATOR.mul(&e1).mul(&e1.inverse());
        assert_eq!(ge, GroupElement::GENERATOR);
    }
}
