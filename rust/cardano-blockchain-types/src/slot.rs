//! Block Slot

use std::{cmp::Ordering, ops::{MulAssign, Sub}};

use crate::conversion::from_saturating;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]

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
        self.0 *= rhs;
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