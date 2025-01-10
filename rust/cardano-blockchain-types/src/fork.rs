//! Fork count is a counter that is incremented every time there is a roll-back in
//! live-chain It is used to help followers determine how far to roll-back to
//! resynchronize without storing full block history. The fork count starts at 1 for live
//! blocks and increments if the live chain tip is purged due to a detected fork, but it
//! does not track the exact number of forks reported by peers.
//!
//! Note: This fork terminology is different from fork in blockchain.

use std::fmt;

use crate::conversion::from_saturating;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
/// Counter that is incremented every time there is a roll-back in live-chain.
pub struct Fork(u64);

impl Fork {
    /// Fork for data that read from the blockchain during a backfill on initial sync
    pub const BACKFILL: Self = Self(1);
    /// Fork count for the first live block.
    pub const FIRST_LIVE: Self = Self(2);
    /// Fork for immutable data. This indicates that there is no roll-back.
    pub const IMMUTABLE: Self = Self(0);

    /// Is the fork for immutable data.
    #[must_use]
    pub fn is_immutable(&self) -> bool {
        self.0 == 0
    }

    /// Is the fork for backfill data.
    #[must_use]
    pub fn is_backfill(&self) -> bool {
        self.0 == 1
    }

    /// Is the fork for live data.
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.0 > 1
    }

    /// Convert an `<T>` to `Fork` (saturate if out of range).
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

    /// Increment the fork count.
    pub fn incr(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Decrement the fork count.
    pub fn decr(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl fmt::Display for Fork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "IMMUTABLE"),
            1 => write!(f, "BACKFILL"),
            // For live forks: 2 maps to LIVE:1, 3 maps to LIVE:2 etc.
            2..=u64::MAX => write!(f, "LIVE:{}", self.0 - 1),
        }
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
