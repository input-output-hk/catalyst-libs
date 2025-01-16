//! Block Slot

use std::{
    cmp::Ordering,
    ops::{MulAssign, Sub},
};

use catalyst_types::conversion::from_saturating;
use num_bigint::{BigInt, Sign};
use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize)]

/// Slot on the blockchain, typically one slot equals one second.  However chain
/// parameters can alter how long a slot is.
pub struct Slot(u64);

impl Slot {
    /// Convert an `<T>` to Slot. (saturate if out of range.)
    pub fn from_saturating<
        T: Copy
            + TryInto<u64>
            + std::ops::Sub<Output = T>
            + std::cmp::PartialOrd<T>
            + num_traits::identities::Zero,
    >(
        value: T,
    ) -> Self {
        let value: u64 = from_saturating(value);
        Self(value)
    }
}

impl From<u64> for Slot {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Slot> for u64 {
    fn from(val: Slot) -> Self {
        val.0
    }
}

impl MulAssign<u64> for Slot {
    fn mul_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_mul(rhs);
    }
}

impl PartialOrd for Slot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Sub for Slot {
    type Output = Slot;

    fn sub(self, rhs: Slot) -> Self::Output {
        Slot(self.0.saturating_sub(rhs.0))
    }
}

impl From<BigInt> for Slot {
    fn from(value: BigInt) -> Self {
        if value.sign() == Sign::Minus {
            Slot(0)
        } else {
            let v: u64 = value.try_into().unwrap_or(u64::MAX);
            Slot::from_saturating(v)
        }
    }
}

impl From<Slot> for BigInt {
    fn from(val: Slot) -> Self {
        BigInt::from(val.0)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::*;

    #[test]
    fn test_from_bigint_to_slot_positive() {
        const N: u64 = 12345;
        let big_int = BigInt::from(N); // positive BigInt
        let slot: Slot = big_int.into();
        assert_eq!(slot.0, N);
    }

    #[test]
    fn test_from_bigint_to_slot_negative() {
        let big_int = BigInt::from(-12345); // negative BigInt
        let slot: Slot = big_int.into();
        assert_eq!(slot.0, 0);
    }

    #[test]
    fn test_from_bigint_to_slot_large_value() {
        let big_int = BigInt::from(u128::MAX); // large BigInt that exceeds u64
        let slot: Slot = big_int.into(); // should saturate to u64::MAX
        assert_eq!(slot.0, u64::MAX);
    }

    #[test]
    fn test_from_slot_to_bigint_positive() {
        const N: u64 = 12345;
        let slot = Slot(N);
        let big_int: BigInt = slot.into(); // should convert back to BigInt
        assert_eq!(big_int, BigInt::from(N));
    }

    #[test]
    fn test_from_slot_to_bigint_zero() {
        const N: u64 = 0;
        let slot = Slot(N);
        let big_int: BigInt = slot.into(); // should convert to BigInt::from(0)
        assert_eq!(big_int, BigInt::from(N));
    }

    #[test]
    fn test_from_slot_to_bigint_large_value() {
        let slot = Slot(u64::MAX);
        let big_int: BigInt = slot.into(); // should convert to BigInt::from(u64::MAX)
        assert_eq!(big_int, BigInt::from(u64::MAX));
    }
}
