//! Cardano chain follow module.

use cardano_blockchain_types::{Fork, MultiEraBlock, Network, Point};
use pallas::network::miniprotocols::txmonitor::{TxBody, TxId};
use tokio::sync::broadcast::{self};
use tracing::{debug, error, warn};

use crate::{
    chain_sync::point_at_tip,
    chain_sync_live_chains::{find_best_fork_block, get_live_block, live_chain_length},
    chain_sync_ready::{block_until_sync_ready, get_chain_update_rx_queue},
    chain_update::{self, ChainUpdate},
    mithril_snapshot::MithrilSnapshot,
    mithril_snapshot_data::latest_mithril_snapshot_id,
    mithril_snapshot_iterator::MithrilSnapshotIterator,
    stats::{self},
    Kind, Statistics,
};

/// The Chain Follower
pub struct ChainFollower {
    /// The Blockchain network we are following.
    chain: Network,
    /// Where we end following.
    end: Point,
    /// Point we processed most recently.
    previous: Point,
    /// Point we are currently in the following process.
    current: Point,
    /// What fork were we last on
    fork: Fork,
    /// Mithril Snapshot
    snapshot: MithrilSnapshot,
    /// Mithril Snapshot Follower
    mithril_follower: Option<MithrilSnapshotIterator>,
    /// Mithril TIP Reached
    mithril_tip: Option<Point>,
    /// Live Block Updates
    sync_updates: broadcast::Receiver<chain_update::Kind>,
}

impl ChainFollower {
    /// Follow a blockchain.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain network to follow.
    /// * `start` - The point or tip to start following from (inclusive).
    /// * `end` - The point or tip to stop following from (inclusive).
    ///
    /// # Returns
    ///
    /// The Chain Follower that will return blocks in the requested range.
    ///
    /// # Notes
    ///
    /// IF end < start, the follower will immediately yield no blocks.
    /// IF end is TIP, then the follower will continue to follow even when TIP is reached.
    /// Otherwise only blocks in the request range will be returned.
    ///
    /// Also, UNLIKE the blockchain itself, the only relevant information is the Slot#.
    /// The Block hash is not considered.
    /// If start is not an exact Slot#, then the NEXT Slot immediately following will be
    /// the first block returned.
    /// If the end is also not an exact Slot# with a block, then the last block will be
    /// the one immediately proceeding it.
    ///
    /// To ONLY follow from TIP, set BOTH start and end to TIP.
    #[must_use]
    pub async fn new(chain: Network, start: Point, end: Point) -> Self {
        let rx = get_chain_update_rx_queue(chain).await;

        ChainFollower {
            chain,
            end,
            previous: Point::UNKNOWN,
            current: start,
            fork: Fork::BACKFILL, // This is correct, because Mithril is Fork 0.
            snapshot: MithrilSnapshot::new(chain),
            mithril_follower: None,
            mithril_tip: None,
            sync_updates: rx,
        }
    }

    /// If we can, get the next update from the mithril snapshot.
    async fn next_from_mithril(&mut self) -> Option<ChainUpdate> {
        loop {
            let current_mithril_tip = latest_mithril_snapshot_id(self.chain).tip();

            // Get previous mithril tip, or set it and return current if a previous does not exist.
            let previous_mithril_tip = self.mithril_tip.get_or_insert_with(|| {
                debug!(
                    mithril_tip = ?current_mithril_tip,
                    "Setting Initial Mithril Tip"
                );
                current_mithril_tip.clone()
            });

            // Return an ImmutableBlockRollForward event as soon as we can after one occurs.
            // This is not an advancement in the followers sequential block iterating state.
            // BUT it is a necessary status to return to a follower, so it can properly handle
            // when immutable state advances (if it so requires)
            if current_mithril_tip != *previous_mithril_tip {
                debug!(
                    new_tip = ?self.mithril_tip,
                    current_tip = ?current_mithril_tip,
                    "Mithril Tip has changed"
                );
                // We have a new mithril tip so report Mithril Tip Roll Forward
                if let Some(block) = self.snapshot.read_block_at(&current_mithril_tip).await {
                    // Update the snapshot in the follower state to the new snapshot.
                    let update = ChainUpdate::new(
                        chain_update::Kind::ImmutableBlockRollForward,
                        false, // Tip is Live chain tip, not Mithril Tip, and this is Mithril Tip.
                        block,
                    );
                    return Some(update);
                }
                // This can only happen if the snapshot does not contain the tip block.
                // So its effectively impossible/unreachable.
                // However, IF it does happen, nothing bad (other than a delay to reporting
                // immutable roll forward) will occur, so we log this impossible
                // error, and continue processing.
                error!(
                    tip = ?self.mithril_tip,
                    current = ?current_mithril_tip,
                    "Mithril Tip Block is not in snapshot. Should not happen."
                );
            }

            if current_mithril_tip <= self.current {
                return None;
            }

            if self.mithril_follower.is_none() {
                self.mithril_follower = self
                    .snapshot
                    .try_read_blocks_from_point(&self.current)
                    .await;
            }

            if let Some(follower) = self.mithril_follower.as_mut() {
                if let Some(next) = follower.next().await {
                    let update = ChainUpdate::new(chain_update::Kind::Block, false, next);
                    return Some(update);
                }

                // Verifying ultra rare scenario of race condition on the mithril snapshot data
                // directory, where the underlying data directory could be no longer accessible
                if !follower.is_valid() {
                    // Set the mithril follower to None and restart the loop
                    warn!("Detected Mithril snapshot data directory race condition, underlying data directory is not accessible anymore: Correcting...");
                    self.mithril_follower = None;
                    continue;
                }
            }
            return None;
        }
    }

    /// If we can, get the next update from the live chain.
    async fn next_from_live_chain(&mut self) -> Option<ChainUpdate> {
        let mut next_block: Option<MultiEraBlock> = None;
        let mut update_type = chain_update::Kind::Block;
        let mut rollback_depth: u64 = 0;

        // Special Case: point = TIP_POINT.  Just return the latest block in the live chain.
        if self.current == Point::TIP {
            next_block = {
                let block = get_live_block(self.chain, &self.current, -1, false)?;
                Some(block)
            };
        }

        // In most cases we will be able to get the next block.
        if next_block.is_none() {
            // If we don't know the previous block, get the block requested.
            let advance = i64::from(!self.previous.is_unknown());
            next_block = get_live_block(self.chain, &self.current, advance, true);
        }

        // If we can't get the next consecutive block, then
        // Get the best previous block.
        if next_block.is_none() {
            debug!("No blocks left in live chain.");

            // IF this is an update still, and not us having caught up, then it WILL be a rollback.
            update_type = chain_update::Kind::Rollback;
            next_block = if let Some((block, depth)) =
                find_best_fork_block(self.chain, &self.current, &self.previous, self.fork)
            {
                debug!("Found fork block: {block}");
                // IF the block is the same as our current previous, there has been no chain
                // advancement, so just return None.
                if block.point().strict_eq(&self.current) {
                    None
                } else {
                    rollback_depth = depth;
                    Some(block)
                }
            } else {
                debug!("No block to find, rewinding to latest mithril tip.");
                let latest_mithril_point = latest_mithril_snapshot_id(self.chain).tip();
                if let Some(block) = MithrilSnapshot::new(self.chain)
                    .read_block_at(&latest_mithril_point)
                    .await
                {
                    rollback_depth = live_chain_length(self.chain) as u64;
                    Some(block)
                } else {
                    return None;
                }
            }
        }

        if let Some(next_block) = next_block {
            // Update rollback stats for the follower if one is reported.
            if update_type == chain_update::Kind::Rollback {
                stats::rollback::rollback(
                    self.chain,
                    stats::rollback::RollbackType::Follower,
                    rollback_depth,
                );
            }

            let tip = point_at_tip(self.chain, &self.current).await;
            let update = ChainUpdate::new(update_type, tip, next_block);
            return Some(update);
        }

        None
    }

    /// Update the current Point, and return `false` if this fails.
    fn update_current(&mut self, update: &ChainUpdate) {
        if update.kind == Kind::ImmutableBlockRollForward {
            // The ImmutableBlockRollForward includes the Mithril TIP Block.
            // Update the mithril_tip state to the point of it.
            self.mithril_tip = Some(update.data.point());
            debug!(mithril_tip=?self.mithril_tip, "Updated followers current Mithril Tip");
            // We DO NOT update anything else for this kind of update, as its informational and
            // does not advance the state of the follower to a new block.
            // It is still a valid update, and so return true, but don't update more state.
            return;
        }
        // Avoids of doing unnecessary clones.
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current = update.block_data().point();
        self.fork = update.block_data().fork();
    }

    /// This is an unprotected version of `next()` which can ONLY be used within this
    /// crate. Its purpose is to allow the chain data to be inspected/validate prior
    /// to unlocking it for general access.
    ///
    /// This function can NOT return None, but that state is used to help process data.
    ///
    /// This function must not be exposed for general use.
    async fn unprotected_next(&mut self) -> Option<ChainUpdate> {
        // We will loop here until we can successfully return a new block
        loop {
            // Try and get the next update from the mithril chain, and return it if we are
            // successful.
            if let Some(update) = self.next_from_mithril().await {
                self.update_current(&update);
                return Some(update);
            }

            // No update from Mithril Data, so try and get one from the live chain.
            if let Some(update) = self.next_from_live_chain().await {
                self.update_current(&update);
                return Some(update);
            }

            // IF we can't get a new block directly from the mithril data, or the live chain, then
            // wait for something to change which might mean we can get the next block.
            // Note, this is JUST a trigger, we don't process based on it other than to allow
            // a blocked follower to continue.
            let changed_data_trigger = self.sync_updates.recv().await;
            match changed_data_trigger {
                Ok(kind) => {
                    // The KIND of event signaling changed data is not important, but we do log it
                    // to help with debugging in case an update stops.
                    debug!("Update kind: {kind}");
                },
                Err(tokio::sync::broadcast::error::RecvError::Lagged(distance)) => {
                    // The update queue is small, its possible that it fills before a task can
                    // read from it, this will cause this Lagged error.
                    // BUT, because we don't care what the event was, this is as good as the missed
                    // event.  Therefore its not an error, and just log it at debug to help with
                    // debugging the logic only.
                    debug!("Lagged by {} updates", distance);
                },
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    // The queue is closed, so we need to return that its no longer possible to
                    // get data from this follower.
                    // This is not an error.
                    return None;
                },
            }
        }
    }

    /// Get the next block from the follower.
    /// Returns NONE is there is no block left to return.
    pub async fn next(&mut self) -> Option<ChainUpdate> {
        // If we aren't syncing TIP, and Current >= End, then return None
        if self.end != Point::TIP && self.current >= self.end {
            return None;
        }

        // Can't follow if SYNC is not ready.
        block_until_sync_ready(self.chain).await;

        // Get next block from the iteration.
        self.unprotected_next().await
    }

    /// Get a single block from the chain by its point.
    ///
    /// If the Point does not point exactly at a block, it will return the next
    /// consecutive block.
    ///
    /// This is a convenience function which just used `ChainFollower` to fetch a single
    /// block.
    pub async fn get_block(chain: Network, point: Point) -> Option<ChainUpdate> {
        // Get the block from the chain.
        // This function suppose to run only once, so the end point
        // can be set to `TIP_POINT`
        let mut follower = Self::new(chain, point, Point::TIP).await;
        follower.next().await
    }

    /// Get the current Immutable and live tips.
    ///
    /// Note, this will block until the chain is synced, ready to be followed.
    pub async fn get_tips(chain: Network) -> (Point, Point) {
        // Can't follow if SYNC is not ready.
        block_until_sync_ready(chain).await;

        let tips = Statistics::tips(chain);

        let mithril_tip = Point::fuzzy(tips.0);
        let live_tip = Point::fuzzy(tips.1);

        (mithril_tip, live_tip)
    }

    /// Schedule a transaction to be posted to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to post the transaction on.
    /// * `txn` - The transaction to be posted.
    ///
    /// # Returns
    ///
    /// `TxId` - The ID of the transaction that was queued.
    #[allow(clippy::unused_async)]
    pub async fn post_txn(chain: Network, txn: TxBody) -> TxId {
        #[allow(clippy::no_effect_underscore_binding)]
        let _unused = chain;
        #[allow(clippy::no_effect_underscore_binding)]
        let _unused = txn;

        "unimplemented".to_string()
    }

    /// Check if a transaction, known by its `TxId`, has been sent to the Peer Node.
    ///
    /// Note, the `TxId` can ONLY be checked for ~6 hrs after it was posted.
    /// After which, it should be on the blockchain, and its the applications job to track
    /// if a transaction made it on-chain or not.
    #[allow(clippy::unused_async)]
    pub async fn txn_sent(chain: Network, id: TxId) -> bool {
        #[allow(clippy::no_effect_underscore_binding)]
        let _unused = chain;
        #[allow(clippy::no_effect_underscore_binding)]
        let _unused = id;

        false
    }
}

// TODO(SJ) - Add a function to check if a transaction is pending, or has been sent to the
// chain.

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_block() -> MultiEraBlock {
        let raw_block = hex::decode(include_str!("./../test_data/shelley.block"))
            .expect("Failed to decode hex block.");

        let pallas_block = pallas::ledger::traverse::MultiEraBlock::decode(raw_block.as_slice())
            .expect("cannot decode block");

        let previous_point = Point::new(
            (pallas_block.slot().checked_sub(1).unwrap()).into(),
            pallas_block
                .header()
                .previous_hash()
                .expect("cannot get previous hash")
                .into(),
        );

        MultiEraBlock::new(
            Network::Preprod,
            raw_block.clone(),
            &previous_point,
            1.into(),
        )
        .expect("cannot create block")
    }

    #[tokio::test]
    async fn test_chain_follower_new() {
        let chain = Network::Mainnet;
        let start = Point::new(100u64.into(), [0; 32].into());
        let end = Point::fuzzy(999u64.into());

        let follower = ChainFollower::new(chain, start.clone(), end.clone()).await;

        assert_eq!(follower.chain, chain);
        assert_eq!(follower.end, end);
        assert_eq!(follower.previous, Point::UNKNOWN);
        assert_eq!(follower.current, start);
        assert_eq!(follower.fork, 1.into());
        assert!(follower.mithril_follower.is_none());
        // assert!(follower.mithril_tip.is_none());
    }

    #[tokio::test]
    async fn test_chain_follower_update_current() {
        let chain = Network::Mainnet;
        let start = Point::new(100u64.into(), [0; 32].into());
        let end = Point::fuzzy(999u64.into());

        let mut follower = ChainFollower::new(chain, start.clone(), end.clone()).await;

        let block_data = mock_block();
        let update = ChainUpdate::new(chain_update::Kind::Block, false, block_data);

        let old_current = follower.current.clone();
        follower.update_current(&update);

        assert_eq!(follower.current, update.block_data().point());
        assert_eq!(follower.previous, old_current);
        assert_eq!(follower.fork, update.block_data().fork());
    }

    #[tokio::test]
    async fn test_chain_follower_update_current_immutable_roll_forward() {
        let chain = Network::Mainnet;
        let start = Point::new(100u64.into(), [0; 32].into());
        let end = Point::fuzzy(999u64.into());

        let mut follower = ChainFollower::new(chain, start.clone(), end.clone()).await;

        let block_data = mock_block();
        let update = ChainUpdate::new(
            chain_update::Kind::ImmutableBlockRollForward,
            false,
            block_data,
        );

        let old_current = follower.current.clone();
        follower.update_current(&update);

        assert_eq!(follower.current, old_current);
    }
}
