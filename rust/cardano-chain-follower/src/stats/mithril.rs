//! Mithril Snapshot Statistics

use cardano_blockchain_types::Slot;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Statistics related to Mithril Snapshots
#[derive(Debug, Default, Clone, Serialize)]
pub struct Mithril {
    /// Number of Mithril Snapshots that have downloaded successfully.
    pub updates: u64,
    /// The Immutable TIP Slot# - Origin = No downloaded snapshot
    pub tip: Slot,
    /// Time we started downloading the current snapshot. 1/1/1970-00:00:00 UTC = Never
    /// downloaded.
    pub dl_start: DateTime<Utc>,
    /// Time we finished downloading the current snapshot. if < `dl_start` its the
    /// previous time we finished.
    pub dl_end: DateTime<Utc>,
    /// Number of times download failed (bad server connection)
    pub dl_failures: u64,
    /// The time the last download took, in seconds.
    pub last_dl_duration: u64,
    /// The size of the download archive, in bytes. (If not started and not ended, current
    /// partial download size).
    pub dl_size: u64,
    /// Extraction start time. 1/1/1970-00:00:00 UTC = Never extracted.
    pub extract_start: DateTime<Utc>,
    /// Extraction end time. if `extract_end` < `extract_start` its the previous time we
    /// finished extracting.
    pub extract_end: DateTime<Utc>,
    /// Number of times extraction failed (bad archive)
    pub extract_failures: u64,
    /// Size of last extracted snapshot, in bytes.
    pub extract_size: u64,
    /// Deduplicated Size vs previous snapshot.
    pub deduplicated_size: u64,
    /// Number of identical files deduplicated from previous snapshot.
    pub deduplicated: u64,
    /// Number of changed files from previous snapshot.
    pub changed: u64,
    /// Number of new files from previous snapshot.
    pub new: u64,
    /// Mithril Certificate Validation Start Time. 1/1/1970-00:00:00 UTC = Never
    /// validated.
    pub validate_start: DateTime<Utc>,
    /// Mithril Certificate Validation End Time. if validate end < validate start its the
    /// previous time we finished validating.
    pub validate_end: DateTime<Utc>,
    /// Number of times validation failed (bad snapshot)
    pub validate_failures: u64,
    /// Blocks that failed to deserialize from the mithril immutable chain.
    pub invalid_blocks: u64,
    /// Download Or Validation Failed
    pub download_or_validation_failed: u64,
    /// Failed to get tip from mithril snapshot.
    pub failed_to_get_tip: u64,
    /// Tip failed to advance
    pub tip_did_not_advance: u64,
    /// Failed to send new tip to updater.
    pub tip_failed_to_send_to_updater: u64,
    /// Failed to activate new snapshot
    pub failed_to_activate_new_snapshot: u64,
}

impl Mithril {
    /// Reset incremental counters in the mithril statistics.
    pub(crate) fn reset(&mut self) {
        self.updates = 0;
        self.dl_failures = 0;
        self.extract_failures = 0;
        self.validate_failures = 0;
        self.invalid_blocks = 0;
    }
}
