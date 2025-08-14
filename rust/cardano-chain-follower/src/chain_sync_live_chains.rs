//! Storage of each Live Chain per Blockchain.

use std::{
    ops::Bound,
    sync::{Arc, LazyLock, RwLock},
    time::Duration,
};

use cardano_blockchain_types::{pallas_primitives, Fork, MultiEraBlock, Network, Point, Slot};
use crossbeam_skiplist::SkipMap;
use rayon::prelude::*;
use tracing::debug;

use crate::{
    error::{Error, Result},
    mithril_snapshot_data::latest_mithril_snapshot_id,
    stats::{self},
};

/// Type we use to manage the Sync Task handle map.
type LiveChainBlockList = SkipMap<Point, MultiEraBlock>;

/// Because we have multi-entry relationships in the live-chain, it need to protect it
/// with a `read/write lock`. The underlying `SkipMap` is still capable of multiple
/// simultaneous reads from multiple threads which is the most common access.
#[derive(Clone)]
struct ProtectedLiveChainBlockList(Arc<RwLock<LiveChainBlockList>>);

/// Handle to the mithril sync thread. One for each Network ONLY.
static LIVE_CHAINS: LazyLock<SkipMap<Network, ProtectedLiveChainBlockList>> =
    LazyLock::new(|| SkipMap::new());

/// Latest TIP received from the Peer Node.
static PEER_TIP: LazyLock<SkipMap<Network, Point>> = LazyLock::new(|| SkipMap::new());

/// Initial slot age to probe.
const INITIAL_SLOT_PROBE_AGE: u64 = 40;

/// Set the last TIP received from the peer.
fn update_peer_tip(
    chain: Network,
    tip: Point,
) {
    PEER_TIP.insert(chain, tip);
}

/// Get the last TIP received from the peer.
/// If the peer tip doesn't exist, get the UNKNOWN point.
pub(crate) fn get_peer_tip(chain: Network) -> Point {
    (*PEER_TIP.get_or_insert(chain, Point::UNKNOWN).value()).clone()
}

/// Number of seconds to wait if we detect a `SyncReady` race condition.
const DATA_RACE_BACKOFF_SECS: u64 = 2;

impl ProtectedLiveChainBlockList {
    /// Create a new instance of the protected Live Chain skip map.
    fn new() -> Self {
        ProtectedLiveChainBlockList(Arc::new(RwLock::new(LiveChainBlockList::new())))
    }

    /// Get the `nth` Live block immediately following the specified block.
    /// If the search is NOT strict, then the point requested is never found.
    /// 0 = The Block immediately after the requested point.
    /// 1+ = The block that follows the block after the requested point
    /// negative = The block before the requested point.
    fn get_block(
        &self,
        point: &Point,
        mut advance: i64,
        strict: bool,
    ) -> Option<MultiEraBlock> {
        let chain = self.0.read().ok()?;

        let mut this = if strict && !point.is_fuzzy() {
            // Strict with concrete point.
            // We can call `get` only for non fuzzy points.
            // Because of the `Point` type equality strictly defined that fuzzy point and non fuzzy
            // point are always not equal, even if they are pointing to the same slot.
            chain.get(point)?
        } else if strict {
            // Strict but fuzzy point.
            // For the fuzzy point make a a fuzzy lookup forwards (including the point).
            // Because of the `Point` type equality strictly defined that fuzzy point and non fuzzy
            // point are always not equal, even if they are pointing to the same slot.
            let found = chain.lower_bound(Bound::Included(point))?;
            // make sure the found point is what we wanted.
            if point.slot_or_default() != found.value().point().slot_or_default() {
                return None;
            }
            found
        } else if advance < 0 {
            // This is a fuzzy lookup backwards.
            advance = advance.saturating_add(1);
            chain.upper_bound(Bound::Excluded(point))?
        } else {
            // This is a fuzzy lookup forwards.
            chain.lower_bound(Bound::Excluded(point))?
        };

        // If we are stepping backwards, look backwards.
        while advance < 0 {
            advance = advance.saturating_add(1);
            this = this.prev()?;
        }

        // If we are stepping forwards, look forwards.
        while advance > 0 {
            advance = advance.saturating_sub(1);
            this = this.next()?;
        }

        // Return the block we found.
        Some(this.value().clone())
    }

    /// Get the earliest block in the Live Chain
    fn get_earliest_block(&self) -> Option<MultiEraBlock> {
        let chain = self.0.read().ok()?;
        let entry = chain.front()?;
        Some(entry.value().clone())
    }

    /// Get the point of the first known block in the Live Chain.
    fn get_first_live_point(live_chain: &LiveChainBlockList) -> Result<Point> {
        let Some(check_first_live_entry) = live_chain.front() else {
            return Err(Error::LiveSync(
                "First Block not found in the Live Chain during Backfill".to_string(),
            ));
        };
        let check_first_live_block = check_first_live_entry.value();
        Ok(check_first_live_block.point())
    }

    /// Get the point of the last known block in the Live Chain.
    fn get_last_live_point(live_chain: &LiveChainBlockList) -> Point {
        let Some(check_last_live_entry) = live_chain.back() else {
            // Its not an error if we can't get a latest block because the chain is empty,
            // so report that we don't know...
            return Point::UNKNOWN;
        };
        let check_last_live_block = check_last_live_entry.value();
        check_last_live_block.point()
    }

    /// Atomic Backfill the chain with the given blocks
    /// Blocks must be sorted in order from earliest to latest.
    /// Final block MUST seamlessly link to the current head of the live chain. (Enforced)
    /// First block MUST seamlessly link to the Tip of the Immutable chain. (Enforced)
    /// The blocks MUST be contiguous and properly self referential.
    /// Note: This last condition is NOT enforced, but must be met or block chain
    /// iteration will fail.
    fn backfill(
        &self,
        chain: Network,
        blocks: &[MultiEraBlock],
    ) -> Result<()> {
        let live_chain = self.0.write().map_err(|_| Error::Internal)?;

        // Make sure our first live block == the last mithril tip.
        // Ensures we are properly connected to the Mithril Chain.
        let first_block_point = blocks
            .first()
            .ok_or(Error::LiveSync("No first block for backfill.".to_string()))?
            .point();
        let latest_mithril_tip = latest_mithril_snapshot_id(chain).tip();
        if !first_block_point.strict_eq(&latest_mithril_tip) {
            return Err(Error::LiveSync(format!(
                "First Block of Live BackFill {first_block_point} MUST be last block of Mithril Snapshot {latest_mithril_tip}."
            )));
        }

        // Get the current Oldest block in the live chain.
        let check_first_live_point = Self::get_first_live_point(&live_chain)?;

        let last_backfill_block = blocks
            .last()
            .ok_or(Error::LiveSync("No last block for backfill.".to_string()))?
            .clone();
        let last_backfill_point = last_backfill_block.point();

        // Make sure the backfill will properly connect the partial Live chain to the Mithril
        // chain.
        if !last_backfill_point.strict_eq(&check_first_live_point) {
            return Err(Error::LiveSync(format!(
                "Last Block of Live BackFill {last_backfill_point} MUST be First block of current Live Chain {check_first_live_point}."
            )));
        }

        // SkipMap is thread-safe, so we can parallel iterate inserting the blocks.
        blocks.par_iter().for_each(|block| {
            let _unused = live_chain.insert(block.point(), block.clone());
        });

        // End of Successful backfill == Reaching TIP, because live sync is always at tip.
        stats::tip_reached(chain);

        Ok(())
    }

    /// Check if the given point is strictly in the live-chain. This means the slot and
    /// block hash MUST be present.
    fn strict_block_lookup(
        live_chain: &LiveChainBlockList,
        point: &Point,
    ) -> bool {
        if let Some(found_block) = live_chain.get(point) {
            return found_block.value().point().strict_eq(point);
        }
        false
    }

    /// Adds a block to the tip of the live chain, and automatically purges blocks that
    /// would be lost due to rollback. Will REFUSE to add a block which does NOT have
    /// a proper "previous" point defined.
    fn add_block_to_tip(
        &self,
        chain: Network,
        block: MultiEraBlock,
        fork_count: &mut Fork,
        tip: Point,
    ) -> Result<()> {
        let live_chain = self.0.write().map_err(|_| Error::Internal)?;

        // Check if the insert is the next logical block in the live chain.
        // Most likely case, so check it first.
        let previous_point = block.previous();
        let last_live_point = Self::get_last_live_point(&live_chain);
        if !previous_point.strict_eq(&last_live_point) {
            // Detected a rollback, so increase the fork count.
            fork_count.incr();
            let mut rollback_size: u64 = 0;

            // We are NOT contiguous, so check if we can become contiguous with a rollback.
            debug!("Detected non-contiguous block, rolling back. Fork: {fork_count}");

            // First check if the previous is >= the earliest block in the live chain.
            // This is because when we start syncing we could rollback earlier than our
            // previously known earliest block.
            // Also check the point we want to link to actually exists.  If either are not true,
            // Then we could be trying to roll back to an earlier block than our earliest known
            // block.
            let check_first_live_point = Self::get_first_live_point(&live_chain)?;
            if (block.point() < check_first_live_point)
                || !Self::strict_block_lookup(&live_chain, &previous_point)
            {
                debug!("Rollback before live chain, clear it.");
                // We rolled back earlier than the current live chain.
                // Purge the entire chain, and just add this one block as the new tip.
                rollback_size = live_chain.len() as u64;
                live_chain.clear();
            } else {
                // If we get here we know for a fact that the previous block exists.
                // Remove the latest live block, and keep removing it until we re-establish
                // connection with the chain sequence.
                // We search backwards because a rollback is more likely in the newest blocks than
                // the oldest.
                while let Some(popped) = live_chain.pop_back() {
                    rollback_size = rollback_size.saturating_add(1);
                    if previous_point.strict_eq(&popped.value().previous()) {
                        // We are now contiguous, so stop purging.
                        break;
                    }
                }
            }

            // Record a rollback statistic (We record the ACTUAL size our rollback effected our
            // internal live chain, not what the node thinks.)
            stats::rollback::rollback(
                chain,
                stats::rollback::RollbackType::LiveChain,
                rollback_size,
            );
        }

        let head_slot = block.point().slot_or_default();

        // Add the block to the tip of the Live Chain.
        let _unused = live_chain.insert(block.point(), block);

        let tip_slot = tip.slot_or_default();
        update_peer_tip(chain, tip);

        // Record the new live chain stats after we add a new block.
        stats::new_live_block(chain, live_chain.len() as u64, head_slot, tip_slot);

        Ok(())
    }

    /// Checks if the point exists in the live chain.
    /// If it does, removes all block preceding it (but not the point itself).
    /// Will refuse to purge if the point is not the TIP of the mithril chain.
    fn purge(
        &self,
        chain: Network,
        point: &Point,
    ) -> Result<()> {
        // Make sure our first live block == the last mithril tip.
        // Ensures we are properly connected to the Mithril Chain.
        // But don't check this if we are about to purge the entire chain.
        // We do this before we bother locking the chain for update.
        if *point != Point::TIP {
            let latest_mithril_tip = latest_mithril_snapshot_id(chain).tip();
            if !point.strict_eq(&latest_mithril_tip) {
                return Err(Error::LiveSync(format!(
                "First Block of Live Purge {point} MUST be last block of Mithril Snapshot {latest_mithril_tip}."
            )));
            }
        }

        let live_chain = self.0.write().map_err(|_| Error::Internal)?;

        // Special Case.
        // If the Purge Point == TIP_POINT, then we purge the entire chain.
        if *point == Point::TIP {
            live_chain.clear();
        } else {
            // If the block we want to purge upto must be in the chain.
            let Some(purge_start_block_entry) = live_chain.get(point) else {
                return Err(Error::LiveSync(format!(
                    "The block to purge to {point} is not in the Live chain."
                )));
            };

            // Make sure the block that IS present, is the actual block, by strict equality.
            if !purge_start_block_entry.value().point().strict_eq(point) {
                return Err(Error::LiveSync(format!(
                "The block to purge to {point} slot is in the live chain, but its hashes do not match."
            )));
            }

            // Purge every block prior to the purge point.
            while let Some(previous_block) = purge_start_block_entry.prev() {
                let _unused = previous_block.remove();
            }

            // Try and FORCE the skip map to reclaim its memory
            crossbeam_epoch::pin().flush();
            crossbeam_epoch::pin().flush();
        }

        Ok(())
    }

    /// Get the current number of blocks in the live chain
    fn len(&self) -> usize {
        if let Ok(chain) = self.0.read() {
            chain.len()
        } else {
            0
        }
    }

    /// Get chain sync intersection points for communicating with peer node.
    fn get_intersect_points(&self) -> Vec<pallas_primitives::types::point::Point> {
        let mut intersect_points = Vec::new();

        let Ok(chain) = self.0.read() else {
            return intersect_points;
        };

        // Add the top 4 blocks as the first points to intersect.
        let Some(entry) = chain.back() else {
            return intersect_points;
        };
        intersect_points.push(entry.value().point().into());
        for _ in 0..2 {
            if let Some(entry) = entry.prev() {
                intersect_points.push(entry.value().point().into());
            } else {
                return intersect_points;
            };
        }

        // Now find points based on an every increasing Slot age.
        let mut slot_age: Slot = INITIAL_SLOT_PROBE_AGE.into();
        let reference_slot = entry.value().point().slot_or_default();
        let mut previous_point = entry.value().point();

        // Loop until we exhaust probe slots, OR we would step past genesis.
        // It is ok because slot implement saturating subtraction.
        #[allow(clippy::arithmetic_side_effects)]
        while slot_age < reference_slot {
            let ref_point = Point::fuzzy(reference_slot - slot_age);
            let Some(entry) = chain.lower_bound(Bound::Included(&ref_point)) else {
                break;
            };
            if entry.value().point() == previous_point {
                break;
            }
            previous_point = entry.value().point();
            intersect_points.push(previous_point.clone().into());
            slot_age *= 2;
        }

        intersect_points
    }

    /// Given a known point on the live chain, and a fork count, find the best block we
    /// have.
    fn find_best_fork_block(
        &self,
        point: &Point,
        previous_point: &Point,
        fork: Fork,
    ) -> Option<(MultiEraBlock, u64)> {
        let mut rollback_depth: u64 = 0;
        let Ok(chain) = self.0.read() else {
            return None;
        };

        // Get the block <= the current slot.
        let ref_point = Point::fuzzy(point.slot_or_default());
        let mut entry = chain.upper_bound(Bound::Included(&ref_point))?;

        let mut this_block = entry.value().clone();
        // Check if the previous block is the one we previously knew, and if so, thats the best
        // block.
        if this_block.point().strict_eq(previous_point) {
            return Some((this_block, rollback_depth));
        }

        // Search backwards for a fork smaller than or equal to the one we know.
        while this_block.fork() > fork {
            rollback_depth = rollback_depth.saturating_add(1);
            entry = entry.prev()?;

            this_block = entry.value().clone();
        }

        Some((this_block, rollback_depth))
    }

    /// Get the point of the block at the head of the live chain.
    fn get_live_head_point(&self) -> Option<Point> {
        let live_chain = self.0.read().map_err(|_| Error::Internal).ok()?;

        let head_point = Self::get_last_live_point(&live_chain);
        if head_point == Point::UNKNOWN {
            return None;
        }

        Some(head_point)
    }
}

/// Get the `LiveChainBlockList` for a particular `Network`.
fn get_live_chain(chain: Network) -> ProtectedLiveChainBlockList {
    let entry = LIVE_CHAINS.get_or_insert_with(chain, || ProtectedLiveChainBlockList::new());

    let value = entry.value();
    value.clone()
}

/// Get the head `Point` currently in the live chain.
pub(crate) fn get_live_head_point(chain: Network) -> Option<Point> {
    let live_chain = get_live_chain(chain);
    live_chain.get_live_head_point()
}

/// Get the Live block relative to the specified point.
/// The starting block must exist if the search is strict.
pub(crate) fn get_live_block(
    chain: Network,
    point: &Point,
    advance: i64,
    strict: bool,
) -> Option<MultiEraBlock> {
    let live_chain = get_live_chain(chain);
    live_chain.get_block(point, advance, strict)
}

/// Get the fill to point for a chain.
///
/// Returns the Point of the block we are filling up-to, and it's fork count.
///
/// Note: It MAY change between calling this function and actually backfilling.
/// This is expected and normal behavior.
pub(crate) async fn get_fill_to_point(chain: Network) -> (Point, u64) {
    let live_chain = get_live_chain(chain);

    loop {
        if let Some(earliest_block) = live_chain.get_earliest_block() {
            return (earliest_block.point(), earliest_block.fork().into());
        }
        // Nothing in the Live chain to sync to, so wait until there is.
        tokio::time::sleep(Duration::from_secs(DATA_RACE_BACKOFF_SECS)).await;
    }
}

/// Insert a block into the live chain (in-order).
/// Can ONLY be used to add a new tip block to the live chain.
/// `rollback_count` should be set to 1 on the very first connection, after that,
/// it is maintained by this function, and MUST not be modified elsewhere.
pub(crate) fn live_chain_add_block_to_tip(
    chain: Network,
    block: MultiEraBlock,
    fork: &mut Fork,
    tip: Point,
) -> Result<()> {
    let live_chain = get_live_chain(chain);
    live_chain.add_block_to_tip(chain, block, fork, tip)
}

/// Backfill the live chain with the block set provided.
pub(crate) fn live_chain_backfill(
    chain: Network,
    blocks: &[MultiEraBlock],
) -> Result<()> {
    let live_chain = get_live_chain(chain);
    live_chain.backfill(chain, blocks)
}

/// Get the length of the live chain.
/// Probably used by debug code only, so its ok if this is not use.
pub(crate) fn live_chain_length(chain: Network) -> usize {
    let live_chain = get_live_chain(chain);
    live_chain.len()
}

/// On an immutable update, purge the live-chain up to the new immutable tip.
/// Will error if the point is not in the Live chain.
pub(crate) fn purge_live_chain(
    chain: Network,
    point: &Point,
) -> Result<()> {
    let live_chain = get_live_chain(chain);
    live_chain.purge(chain, point)
}

/// Get intersection points to try and find best point to connect to the node on
/// reconnect.
pub(crate) fn get_intersect_points(chain: Network) -> Vec<pallas_primitives::types::point::Point> {
    let live_chain = get_live_chain(chain);
    live_chain.get_intersect_points()
}

/// Find best block from a fork relative to a point.
pub(crate) fn find_best_fork_block(
    chain: Network,
    point: &Point,
    previous_point: &Point,
    fork: Fork,
) -> Option<(MultiEraBlock, u64)> {
    let live_chain = get_live_chain(chain);
    live_chain.find_best_fork_block(point, previous_point, fork)
}
