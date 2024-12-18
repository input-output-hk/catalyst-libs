//! Fork count is a counter that is incremented every time there is a roll-back in
//! live-chain It is used to help followers determine how far to roll-back to
//! resynchronize without storing full block history. The fork count starts at 1 for live
//! blocks and increments if the live chain tip is purged due to a detected fork, but it
//! does not track the exact number of forks reported by peers.
//!
//! Note: This fork terminology is different from fork in blockchain.

use crate::conversion::from_saturating;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// Counter that is incremented every time there is a roll-back in live-chain
pub struct Fork(u64);

impl Fork {
    /// Convert an `<T>` to Fork. (saturate if out of range.)
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

    /// increment the fork count.
    pub fn incr(&mut self) {
        self.0 += 1;
    }
}

impl From<u64> for Fork {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Fork> for u64 {
    fn from(val: Fork) -> Self {
        val.0
    }
}
