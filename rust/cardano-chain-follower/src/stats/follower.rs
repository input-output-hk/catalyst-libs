//! Individual Follower Statistics.

use cardano_blockchain_types::Slot;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Individual Follower stats
#[derive(Debug, Default, Clone, Serialize)]
pub struct Follower {
    /// Synthetic follower connection ID
    pub id: u64,
    /// Starting slot for this follower (0 = Start at Genesis Block for the chain).
    pub start: Slot,
    /// Current slot for this follower.
    pub current: Slot,
    /// Target slot for this follower (MAX U64 == Follow Tip Forever).
    pub end: Slot,
    /// Current Sync Time.
    pub sync_start: DateTime<Utc>,
    /// When this follower reached TIP or its destination slot.
    pub sync_end: Option<DateTime<Utc>>,
}
