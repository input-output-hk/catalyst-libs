//! Rollback statistics.

use std::sync::{Arc, LazyLock, RwLock};

use cardano_blockchain_types::Network;
use dashmap::DashMap;
use serde::Serialize;
use strum::{EnumIter, IntoEnumIterator};
use tracing::error;

/// Statistics related to a single depth of rollback
#[derive(Debug, Default, Clone, Serialize)]
pub struct Rollback {
    /// How deep was the rollback from tip.
    pub depth: u64,
    /// How many times has a rollback been this deep.
    pub count: u64,
}

/// Statistics for all our known rollback types
/// Rollback Vec is sorted by depth, ascending.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Rollbacks {
    /// These are the ACTUAL rollbacks we did on our live-chain in memory.
    pub live: Vec<Rollback>,
    /// These are the rollbacks reported by the Peer Node, which may not == an actual
    /// rollback on our internal live chain.
    pub peer: Vec<Rollback>,
    /// These are the rollbacks synthesized for followers, based on their reading of the
    /// chain tip.
    pub follower: Vec<Rollback>,
}

/// The types of rollbacks we track for a chain.
#[derive(EnumIter, Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum RollbackType {
    /// Rollback on the in-memory live chain.
    LiveChain,
    /// Rollback signaled by the peer.
    Peer,
    /// Rollback synthesized for the Follower.
    Follower,
}

/// Individual rollback records.
type RollbackRecords = DashMap<u64, Rollback>;
/// Rollback Records per rollback type.
type RollbackTypeMap = DashMap<RollbackType, Arc<RwLock<RollbackRecords>>>;
/// Record of rollbacks.
type RollbackMap = DashMap<Network, RollbackTypeMap>;

/// Statistics of rollbacks detected per chain.
static ROLLBACKS_MAP: LazyLock<RollbackMap> = LazyLock::new(|| {
    let map = RollbackMap::new();
    for network in Network::iter() {
        let type_map = RollbackTypeMap::new();
        for rollback in RollbackType::iter() {
            type_map.insert(rollback, Arc::new(RwLock::new(RollbackRecords::new())));
        }
        map.insert(network, type_map);
    }
    map
});

/// Get the actual rollback map for a chain.
fn lookup_rollback_map(
    chain: Network, rollback: RollbackType,
) -> Option<Arc<RwLock<RollbackRecords>>> {
    let Some(chain_rollback_map) = ROLLBACKS_MAP.get(&chain) else {
        error!("Rollback stats SHOULD BE exhaustively pre-allocated.");
        return None;
    };
    let chain_rollback_map = chain_rollback_map.value();

    let Some(rollback_map) = chain_rollback_map.get(&rollback) else {
        error!("Rollback stats SHOULD BE exhaustively pre-allocated.");
        return None;
    };
    let rollback_map = rollback_map.value();

    Some(rollback_map.clone())
}

/// Extract the current rollback stats as a vec.
pub(crate) fn rollbacks(chain: Network, rollback: RollbackType) -> Vec<Rollback> {
    let Some(rollback_map) = lookup_rollback_map(chain, rollback) else {
        return Vec::new();
    };

    let Ok(rollback_values) = rollback_map.read() else {
        error!("Rollback stats LOCK Poisoned, should not happen.");
        return vec![];
    };

    let mut rollbacks = Vec::new();

    // Get all the rollback stats.
    for stat in rollback_values.iter() {
        rollbacks.push(stat.value().clone());
    }

    rollbacks
}

/// Reset ALL the rollback stats for a given blockchain.
pub(crate) fn rollbacks_reset(chain: Network, rollback: RollbackType) -> Vec<Rollback> {
    let Some(rollback_map) = lookup_rollback_map(chain, rollback) else {
        return Vec::new();
    };

    let Ok(rollbacks) = rollback_map.write() else {
        error!("Rollback stats LOCK Poisoned, should not happen.");
        return vec![];
    };

    rollbacks.clear();

    Vec::new()
}

/// Count a rollback
pub(crate) fn rollback(chain: Network, rollback: RollbackType, depth: u64) {
    let Some(rollback_map) = lookup_rollback_map(chain, rollback) else {
        return;
    };

    let Ok(rollbacks) = rollback_map.write() else {
        error!("Rollback stats LOCK Poisoned, should not happen.");
        return;
    };

    let mut value = match rollbacks.get(&depth) {
        Some(value_entry) => (*value_entry.value()).clone(),
        None => Rollback { depth, count: 0 },
    };

    value.count = value.count.saturating_add(1);

    let _unused = rollbacks.insert(depth, value);
}
