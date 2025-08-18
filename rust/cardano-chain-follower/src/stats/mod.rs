//! Cardano Chain Follower Statistics

pub mod follower;
pub mod live_chain;
pub mod mithril;
pub mod rollback;
pub mod thread;

use std::sync::{Arc, LazyLock, RwLock};

use cardano_blockchain_types::{Network, Slot};
use chrono::Utc;
use dashmap::DashMap;
use rollback::{rollbacks, rollbacks_reset, RollbackType};
use serde::Serialize;
use thread::ThreadStat;
use tracing::error;

use crate::stats::{live_chain::Live, mithril::Mithril};

// -------- GENERAL STATISTIC TRACKING

/// Statistics for a single follower network.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Statistics {
    /// Statistics related to the live connection to the blockchain.
    pub live: Live,
    /// Statistics related to the mithril certified blockchain archive.
    pub mithril: Mithril,
    /// Statistics related to the threads.
    pub thread_stats: DashMap<String, thread::ThreadStat>,
}

/// Type we use to manage the Sync Task handle map.
type StatsMap = DashMap<Network, Arc<RwLock<Statistics>>>;
/// The statistics being maintained per chain.
static STATS_MAP: LazyLock<StatsMap> = LazyLock::new(StatsMap::default);

/// Get the stats for a particular chain.
fn lookup_stats(chain: Network) -> Arc<RwLock<Statistics>> {
    match STATS_MAP.get(&chain) {
        Some(chain_entry) => chain_entry.value().clone(),
        None => {
            let res = Arc::new(RwLock::new(Statistics::default()));
            STATS_MAP.insert(chain, res.clone());
            res
        },
    }
}

impl Statistics {
    /// Get a new statistics struct for a given blockchain network.
    #[must_use]
    pub fn new(chain: Network) -> Self {
        let stats = lookup_stats(chain);

        let Ok(chain_stats) = stats.read() else {
            return Statistics::default();
        };

        let mut this_stats = chain_stats.clone();
        // Set the current rollback stats.
        this_stats.live.rollbacks.live = rollbacks(chain, RollbackType::LiveChain);
        this_stats.live.rollbacks.peer = rollbacks(chain, RollbackType::Peer);
        this_stats.live.rollbacks.follower = rollbacks(chain, RollbackType::Follower);

        this_stats
    }

    /// Reset the incremental counters in a stats record.
    fn reset_stats(&mut self) {
        self.live.reset();
        self.mithril.reset();
    }

    /// Get the current tips of the immutable chain and live chain.
    pub(crate) fn tips(chain: Network) -> (Slot, Slot) {
        let zero_slot = Slot::from_saturating(0);
        let stats = lookup_stats(chain);

        let Ok(chain_stats) = stats.read() else {
            return (zero_slot, zero_slot);
        };

        (chain_stats.mithril.tip, chain_stats.live.head_slot)
    }

    /// Reset amd return cumulative counters contained in the statistics.
    #[must_use]
    pub fn reset(chain: Network) -> Self {
        let stats = lookup_stats(chain);

        let Ok(mut chain_stats) = stats.write() else {
            return Statistics::default();
        };

        chain_stats.reset_stats();

        let mut this_stats = chain_stats.clone();
        // Reset the current rollback stats.
        this_stats.live.rollbacks.live = rollbacks_reset(chain, RollbackType::LiveChain);
        this_stats.live.rollbacks.peer = rollbacks_reset(chain, RollbackType::Peer);
        this_stats.live.rollbacks.follower = rollbacks_reset(chain, RollbackType::Follower);

        this_stats
    }

    /// Return the statistics formatted as JSON
    #[must_use]
    pub fn as_json(
        &self,
        pretty: bool,
    ) -> String {
        let json = if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        };
        match json {
            Ok(json) => json,
            Err(error) => {
                error!("{:?}", error);
                String::new()
            },
        }
    }
}

/// Count the invalidly deserialized blocks
#[allow(dead_code)]
pub(crate) fn stats_invalid_block(
    chain: Network,
    immutable: bool,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if immutable {
        chain_stats.mithril.invalid_blocks = chain_stats.mithril.invalid_blocks.saturating_add(1);
    } else {
        chain_stats.live.invalid_blocks = chain_stats.live.invalid_blocks.saturating_add(1);
    }
}

/// Count the validly deserialized blocks
pub(crate) fn new_live_block(
    chain: Network,
    total_live_blocks: u64,
    head_slot: Slot,
    tip_slot: Slot,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.live.new_blocks = chain_stats.live.new_blocks.saturating_add(1);
    chain_stats.live.blocks = total_live_blocks;
    chain_stats.live.head_slot = head_slot;
    chain_stats.live.tip = tip_slot;
}

/// Track the end of the current mithril update
pub(crate) fn new_mithril_update(
    chain: Network,
    mithril_tip: Slot,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.mithril.updates = chain_stats.mithril.updates.saturating_add(1);
    chain_stats.mithril.tip = mithril_tip;
}

/// Track the current total live blocks count
pub(crate) fn new_live_total_blocks(
    chain: Network,
    total_live_blocks: u64,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.live.blocks = total_live_blocks;
}

/// When did we start the backfill.
pub(crate) fn backfill_started(chain: Network) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    // If we start another backfill, then that means the previous backfill failed, so record
    // it.
    if chain_stats.live.backfill_start.is_some() {
        chain_stats.live.backfill_failures = chain_stats.live.backfill_failures.saturating_add(1);
        chain_stats.live.backfill_failure_time = chain_stats.live.backfill_start;
    }

    chain_stats.live.backfill_start = Some(Utc::now());
}

/// When did we start the backfill.
pub(crate) fn backfill_ended(
    chain: Network,
    backfill_size: u64,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.live.backfill_size = backfill_size;
    chain_stats.live.backfill_end = Some(Utc::now());
}

/// Track statistics about connections to the cardano peer node.
pub(crate) fn peer_connected(
    chain: Network,
    active: bool,
    peer_address: &str,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if active {
        chain_stats.live.reconnects = chain_stats.live.reconnects.saturating_add(1);
        chain_stats.live.last_connect = Utc::now();
        chain_stats.live.last_connected_peer = peer_address.to_string();
    } else {
        chain_stats.live.last_disconnect = Utc::now();
        chain_stats.live.last_disconnected_peer = peer_address.to_string();
    }

    chain_stats.live.connected = active;
}

/// Record when we started syncing
pub(crate) fn sync_started(chain: Network) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.live.sync_start = Utc::now();
}

/// Record when we first reached tip. This can safely be called multiple times.
/// Except for overhead, only the first call will actually record the time.
pub(crate) fn tip_reached(chain: Network) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if chain_stats.live.sync_end.is_none() {
        chain_stats.live.sync_end = Some(Utc::now());
    }
}

/// Record that a Mithril snapshot Download has started.
pub(crate) fn mithril_dl_started(chain: Network) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.mithril.dl_start = Utc::now();
}

/// Record when DL finished, if it fails, set size to None, otherwise the size of the
/// downloaded file.
pub(crate) fn mithril_dl_finished(
    chain: Network,
    dl_size: Option<u64>,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    #[allow(clippy::cast_sign_loss)] // Its OK to cast the i64 to u64 because we clamped it.
    if let Some(dl_size) = dl_size {
        chain_stats.mithril.dl_end = Utc::now();
        chain_stats.mithril.dl_size = dl_size;
        let last_dl_duration = chain_stats
            .mithril
            .dl_end
            .signed_duration_since(chain_stats.mithril.dl_start);
        chain_stats.mithril.last_dl_duration =
            last_dl_duration.num_seconds().clamp(0, i64::MAX) as u64;
    } else {
        chain_stats.mithril.dl_failures = chain_stats.mithril.dl_failures.saturating_add(1);
    }
}

/// Record that extracting the mithril snapshot archive has started.
pub(crate) fn mithril_extract_started(chain: Network) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats.mithril.extract_start = Utc::now();
}

/// Record when DL finished, if it fails, set size to None, otherwise the size of the
/// downloaded file.
pub(crate) fn mithril_extract_finished(
    chain: Network,
    extract_size: Option<u64>,
    deduplicated_size: u64,
    deduplicated_files: u64,
    changed_files: u64,
    new_files: u64,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if let Some(extract_size) = extract_size {
        chain_stats.mithril.extract_end = Utc::now();
        chain_stats.mithril.extract_size = extract_size;
        chain_stats.mithril.deduplicated_size = deduplicated_size;
        chain_stats.mithril.deduplicated = deduplicated_files;
        chain_stats.mithril.changed = changed_files;
        chain_stats.mithril.new = new_files;
    } else {
        chain_stats.mithril.extract_failures =
            chain_stats.mithril.extract_failures.saturating_add(1);
    }
}

/// State of the Mithril cert validation.
#[derive(Copy, Clone)]
pub(crate) enum MithrilValidationState {
    /// Validation Started
    Start,
    /// Validation Failed
    Failed,
    /// Validation Finished
    Finish,
}

/// Record when Mithril Cert validation starts, ends or fails).
pub(crate) fn mithril_validation_state(
    chain: Network,
    mithril_state: MithrilValidationState,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    match mithril_state {
        MithrilValidationState::Start => chain_stats.mithril.validate_start = Utc::now(),
        MithrilValidationState::Failed => {
            chain_stats.mithril.validate_failures =
                chain_stats.mithril.validate_failures.saturating_add(1);
        },
        MithrilValidationState::Finish => chain_stats.mithril.validate_end = Utc::now(),
    }
}

/// Mithril Sync Failures.
#[derive(Copy, Clone)]
pub(crate) enum MithrilSyncFailures {
    /// Download Or Validation Failed
    DownloadOrValidation,
    /// Failed to get tip from mithril snapshot.
    FailedToGetTip,
    /// Tip failed to advance
    TipDidNotAdvance,
    /// Failed to send new tip to updater.
    TipFailedToSendToUpdater,
    /// Failed to activate new snapshot
    FailedToActivateNewSnapshot,
}

/// Record when Mithril Cert validation starts, ends or fails).
pub(crate) fn mithril_sync_failure(
    chain: Network,
    failure: MithrilSyncFailures,
) {
    let stats = lookup_stats(chain);

    let Ok(mut chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    match failure {
        MithrilSyncFailures::DownloadOrValidation => {
            chain_stats.mithril.download_or_validation_failed = chain_stats
                .mithril
                .download_or_validation_failed
                .saturating_add(1);
        },
        MithrilSyncFailures::FailedToGetTip => {
            chain_stats.mithril.failed_to_get_tip =
                chain_stats.mithril.failed_to_get_tip.saturating_add(1);
        },
        MithrilSyncFailures::TipDidNotAdvance => {
            chain_stats.mithril.tip_did_not_advance =
                chain_stats.mithril.tip_did_not_advance.saturating_add(1);
        },
        MithrilSyncFailures::TipFailedToSendToUpdater => {
            chain_stats.mithril.tip_failed_to_send_to_updater = chain_stats
                .mithril
                .tip_failed_to_send_to_updater
                .saturating_add(1);
        },
        MithrilSyncFailures::FailedToActivateNewSnapshot => {
            chain_stats.mithril.failed_to_activate_new_snapshot = chain_stats
                .mithril
                .failed_to_activate_new_snapshot
                .saturating_add(1);
        },
    }
}

// ----------------- THREAD STATISTICS-------------------

/// Initialize a thread statistic with the given name.
/// If it is service thread, mark it as such.
pub(crate) fn start_thread(
    chain: Network,
    name: &str,
    is_service: bool,
) {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    chain_stats
        .thread_stats
        .insert(name.to_string(), ThreadStat::start_thread(is_service));
}

/// Stop the thread with the given name.
pub(crate) fn stop_thread(
    chain: Network,
    name: &str,
) {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if let Some(thread_stat) = chain_stats.thread_stats.get(name) {
        thread_stat.stop_thread();
    };
}

/// Resume the thread with the given name.
pub(crate) fn resume_thread(
    chain: Network,
    name: &str,
) {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if let Some(thread_stat) = chain_stats.thread_stats.get(name) {
        thread_stat.resume_thread();
    };
}

/// Pause the thread with the given name.
pub(crate) fn pause_thread(
    chain: Network,
    name: &str,
) {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return;
    };

    if let Some(thread_stat) = chain_stats.thread_stats.get(name) {
        thread_stat.pause_thread();
    };
}

/// Get the thread statistic with the given name.
#[allow(dead_code)]
pub fn thread_stat(
    chain: Network,
    name: &str,
) -> Option<ThreadStat> {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        // Worst case if this fails (it never should) is we stop updating stats.
        error!("Stats RwLock should never be able to error.");
        return None;
    };

    chain_stats.thread_stats.get(name).map(|stat| stat.clone())
}

/// Get the names of all the thread statistics.
#[allow(dead_code)]
pub fn thread_stat_names(chain: Network) -> Vec<String> {
    let stats = lookup_stats(chain);

    let Ok(chain_stats) = stats.write() else {
        error!("Stats RwLock should never be able to error.");
        return Vec::new();
    };

    chain_stats
        .thread_stats
        .iter()
        .map(|entry| entry.key().clone())
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_mithril_reset() {
        let mut mithril = Mithril {
            updates: 10,
            dl_failures: 5,
            extract_failures: 3,
            validate_failures: 2,
            invalid_blocks: 1,
            ..Default::default()
        };
        mithril.reset();
        assert_eq!(mithril.updates, 0);
        assert_eq!(mithril.dl_failures, 0);
        assert_eq!(mithril.extract_failures, 0);
        assert_eq!(mithril.validate_failures, 0);
        assert_eq!(mithril.invalid_blocks, 0);
    }

    #[test]
    fn test_live_reset() {
        let mut live = Live {
            new_blocks: 10,
            reconnects: 5,
            invalid_blocks: 3,
            ..Default::default()
        };
        live.reset();
        assert_eq!(live.new_blocks, 0);
        assert_eq!(live.reconnects, 0);
        assert_eq!(live.invalid_blocks, 0);
    }

    #[test]
    fn test_statistics_reset_stats() {
        let mut stats = Statistics::default();
        stats.live.new_blocks = 10;
        stats.mithril.updates = 5;
        stats.reset_stats();
        assert_eq!(stats.live.new_blocks, 0);
        assert_eq!(stats.mithril.updates, 0);
    }

    #[test]
    fn test_statistics_as_json() {
        let stats = Statistics::default();
        let json = stats.as_json(true);
        assert!(json.contains("\"blocks\": 0"));
        assert!(json.contains("\"updates\": 0"));
    }

    #[test]
    fn test_new_live_block() {
        let network = Network::Preprod;
        new_live_block(network, 100, 50.into(), 200.into());
        let stats = lookup_stats(network);
        let stats = stats.read().unwrap();
        assert_eq!(stats.live.blocks, 100);
        assert_eq!(stats.live.head_slot, 50.into());
        assert_eq!(stats.live.tip, 200.into());
    }

    #[test]
    fn test_mithril_dl_started() {
        let network = Network::Preprod;
        mithril_dl_started(network);
        let stats = lookup_stats(network);
        let stats = stats.read().unwrap();
        assert!(stats.mithril.dl_start <= Utc::now());
    }
}
