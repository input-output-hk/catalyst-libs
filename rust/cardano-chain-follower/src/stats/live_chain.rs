//! Live Blockchain Statistics.

use cardano_blockchain_types::Slot;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{follower::Follower, rollback::Rollbacks};

/// Statistics related to the live blockchain
#[derive(Debug, Default, Clone, Serialize)]
pub struct Live {
    /// The Time that synchronization to this blockchain started
    pub sync_start: DateTime<Utc>,
    /// The Time that synchronization to this blockchain was complete up-to-tip. None =
    /// Not yet synchronized.
    pub sync_end: Option<DateTime<Utc>>,
    /// When backfill started
    pub backfill_start: Option<DateTime<Utc>>,
    /// Backfill size to achieve synchronization. (0 before sync completed)
    pub backfill_size: u64,
    /// When backfill ended
    pub backfill_end: Option<DateTime<Utc>>,
    /// Backfill Failures
    pub backfill_failures: u64,
    /// The time of the last backfill failure
    pub backfill_failure_time: Option<DateTime<Utc>>,
    /// Current Number of Live Blocks
    pub blocks: u64,
    /// The current head of the live chain slot#
    pub head_slot: Slot,
    /// The current live tip slot# as reported by the peer.
    pub tip: Slot,
    /// Number of times we connected/re-connected to the Node.
    pub reconnects: u64,
    /// Last reconnect time,
    pub last_connect: DateTime<Utc>,
    /// Last reconnect time,
    pub last_connected_peer: String,
    /// Last disconnect time,
    pub last_disconnect: DateTime<Utc>,
    /// Last disconnect time,
    pub last_disconnected_peer: String,
    /// Is there an active connection to the node
    pub connected: bool,
    /// Rollback statistics.
    pub rollbacks: Rollbacks,
    /// New blocks read from blockchain.
    pub new_blocks: u64,
    /// Blocks that failed to deserialize from the blockchain.
    pub invalid_blocks: u64,
    /// Active Followers (range and current depth)
    pub follower: Vec<Follower>,
}

impl Live {
    /// Reset incremental counters in the live statistics.
    pub(crate) fn reset(&mut self) {
        self.new_blocks = 0;
        self.reconnects = 0;
        self.invalid_blocks = 0;
    }
}
