//! Multi Era CBOR Encoded Block Data
//!
//! Data about how the block/transactions can be encoded is found here:
//! <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl>
//!
//! DO NOT USE the documentation/cddl definitions from the head of this repo because it
//! currently lacks most of the documentation needed to understand the format and is also
//! incorrectly generated and contains errors that will be difficult to discern.

use std::{cmp::Ordering, fmt::Display, sync::Arc};

use anyhow::bail;
use ed25519_dalek::VerifyingKey;
use ouroboros::self_referencing;
use pallas::ledger::traverse::MultiEraTx;
use tracing::debug;

use crate::{
    auxdata::{
        block::BlockAuxData, metadatum_label::MetadatumLabel, metadatum_value::MetadatumValue,
    },
    fork::Fork,
    network::Network,
    point::Point,
    txn_index::TxnIndex,
    txn_witness::{TxnWitness, VKeyHash},
    Slot,
};

/// Self-referencing CBOR encoded data of a multi-era block.
/// Note: The fields in the original struct can not be accessed directly
/// The builder creates accessor methods which are called
/// `borrow_raw_data()` and `borrow_block()`
#[self_referencing]
#[derive(Debug)]
struct SelfReferencedMultiEraBlock {
    /// The CBOR encoded data of a multi-era block.
    raw_data: Vec<u8>,

    /// The decoded multi-era block.
    /// References the `raw_data` field.
    #[borrows(raw_data)]
    #[covariant]
    block: pallas::ledger::traverse::MultiEraBlock<'this>,
}

/// Multi-era block - inner.
#[derive(Debug)]
struct MultiEraBlockInner {
    /// What blockchain network was the block produced on.
    network: Network,
    /// The Point on the blockchain this block can be found.
    point: Point,
    /// The previous point on the blockchain before this block.
    /// When the current point is Genesis, so is the previous.
    previous: Point,
    /// The decoded multi-era block.
    data: SelfReferencedMultiEraBlock,
    /// Decoded Metadata in the transactions in the block.
    aux_data: BlockAuxData,
    /// A map of public key hashes to the public key and transaction numbers they are in.
    #[allow(dead_code)]
    witness_map: Option<TxnWitness>,
}

/// Multi-era block.
#[derive(Clone, Debug)]
pub struct MultiEraBlock {
    /// What fork is the block on?
    /// This is NOT part of the inner block, because it is not to be protected by the Arc.
    /// It can change at any time due to rollbacks detected on the live-chain.
    /// This means that any holder of a `MultiEraBlock` will have the actual fork their
    /// block was on when they read it, the live-chain code can modify the actual fork
    /// count at any time without that impacting consumers processing the data.
    /// The fork count itself is used so an asynchronous follower can properly work out
    /// how far to roll back on the live-chain in order to resynchronize, without
    /// keeping a full state of processed blocks.
    /// Followers, simply need to step backwards on the live chain until they find the
    /// previous block they followed, or reach a fork that is <= the fork of the
    /// previous block they followed. They can then safely re-follow from that earlier
    /// point, with full integrity. fork is 0 on any immutable block.
    /// It starts at 1 for live blocks, and is only incremented if the live-chain tip is
    /// purged because of a detected fork based on data received from the peer node.
    /// It does NOT count the strict number of forks reported by the peer node.
    fork: Fork,
    /// The Immutable decoded data about the block itself.
    inner: Arc<MultiEraBlockInner>,
}

impl MultiEraBlock {
    /// Creates a new `MultiEraBlockData` from the given bytes.
    ///
    /// # Errors
    ///
    /// If the given bytes cannot be decoded as a multi-era block, an error is returned.
    pub fn new(
        network: Network, raw_data: Vec<u8>, previous: &Point, fork: Fork,
    ) -> anyhow::Result<Self> {
        let builder = SelfReferencedMultiEraBlockTryBuilder {
            raw_data,
            block_builder: |raw_data| -> Result<_, anyhow::Error> {
                Ok(pallas::ledger::traverse::MultiEraBlock::decode(raw_data)?)
            },
        };
        let self_ref_block = builder.try_build()?;
        let decoded_block = self_ref_block.borrow_block();

        let witness_map = TxnWitness::new(&decoded_block.txs()).ok();

        let slot = decoded_block.slot();

        let point = Point::new(slot.into(), decoded_block.hash().into());

        let byron_block = matches!(
            decoded_block,
            pallas::ledger::traverse::MultiEraBlock::Byron(_)
        );

        // Validate that the Block point is valid.
        if !previous.is_origin() {
            // Every 21600 Blocks, Byron Era has duplicated sequential slot#'s.
            // So this filters them out from the sequential point check.
            // The Hash chain is still checked.
            if (!byron_block || ((slot % 21600) != 0)) && *previous >= slot {
                bail!("Previous slot is not less than current slot:{slot}");
            }

            // Special case, when the previous block is actually UNKNOWN, we can't check it.
            if !previous.is_unknown()
                    // Otherwise, we make sure the hash chain is intact
                    && previous != &decoded_block.header().previous_hash()
            {
                debug!("{}, {:?}", previous, decoded_block.header().previous_hash());

                bail!("Previous Block Hash mismatch with block");
            }
        }

        let aux_data = decoded_block.try_into()?;

        Ok(Self {
            fork,
            inner: Arc::new(MultiEraBlockInner {
                network,
                point,
                previous: previous.clone(),
                data: self_ref_block,
                aux_data,
                witness_map,
            }),
        })
    }

    /// Remake the block on a new fork.
    pub fn set_fork(&mut self, fork: Fork) {
        self.fork = fork;
    }

    /// Decodes the data into a multi-era block.
    ///
    /// # Returns
    ///
    /// The decoded block data, which can easily be processed by a consumer.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn decode(&self) -> &pallas::ledger::traverse::MultiEraBlock {
        self.inner.data.borrow_block()
    }

    /// Decodes the data into a multi-era block.
    ///
    /// # Returns
    ///
    /// The raw byte data of the block.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn raw(&self) -> &Vec<u8> {
        self.inner.data.borrow_raw_data()
    }

    /// Returns the block point of this block.
    ///
    /// # Returns
    ///
    /// The block point of this block.
    #[must_use]
    pub fn point(&self) -> Point {
        self.inner.point.clone()
    }

    /// Returns the block point of the previous block.
    ///
    /// # Returns
    ///
    /// The previous blocks `Point`
    #[must_use]
    pub fn previous(&self) -> Point {
        self.inner.previous.clone()
    }

    /// Is the block data immutable on-chain.
    ///
    /// Immutable blocks are by-definition those that exist in the Mithril Snapshot
    /// (Immutable Database) of the Node.
    ///
    /// # Returns
    ///
    /// `true` if the block is immutable, `false` otherwise.
    #[must_use]
    pub fn is_immutable(&self) -> bool {
        self.fork == 0.into()
    }

    /// What fork is the block from.
    ///
    /// The fork is a synthetic number that represents how many rollbacks have been
    /// detected in the running chain.  The fork is:
    /// - 0 - for all immutable data;
    /// - 1 - for any data read from the blockchain during a *backfill* on initial sync
    /// - 2+ - for each subsequent rollback detected while reading live blocks.
    ///
    /// # Returns
    ///
    /// The fork the block was found on.
    #[must_use]
    pub fn fork(&self) -> Fork {
        self.fork
    }

    /// What blockchain network was the block from
    ///
    /// # Returns
    ///
    /// - The network that this block originated on.
    #[must_use]
    pub fn network(&self) -> Network {
        self.inner.network
    }

    /// Get The Metadata fora a transaction and known label from the block
    ///
    /// # Parameters
    ///
    /// - `txn_idx` - Index of the Transaction in the Block
    /// - `label` - The label of the transaction
    ///
    /// # Returns
    ///
    /// - Metadata for the given label in the transaction.
    /// - Or None if the label requested isn't present.
    #[must_use]
    pub fn txn_metadata(
        &self, txn_idx: TxnIndex, label: MetadatumLabel,
    ) -> Option<&MetadatumValue> {
        let txn = self.inner.aux_data.get(txn_idx)?;
        txn.metadata(label)
    }

    /// Returns the witness map for the block.
    pub(crate) fn witness_map(&self) -> Option<&TxnWitness> {
        self.inner.witness_map.as_ref()
    }

    /// If the witness exists for a given transaction then return its public key.
    #[must_use]
    pub fn witness_for_tx(&self, vkey_hash: &VKeyHash, tx_num: TxnIndex) -> Option<VerifyingKey> {
        if let Some(witnesses) = self.witness_map() {
            if witnesses.check_witness_in_tx(vkey_hash, tx_num) {
                return witnesses.get_witness_vkey(vkey_hash);
            }
        }

        None
    }

    /// Returns a list of transactions withing this block.
    #[must_use]
    pub fn txs(&self) -> Vec<MultiEraTx> {
        self.decode().txs()
    }

    /// Returns an iterator over `(TxnIndex, MultiEraTx)` pair.
    pub fn enumerate_txs(&self) -> impl Iterator<Item = (TxnIndex, MultiEraTx)> {
        self.txs()
            .into_iter()
            .enumerate()
            .map(|(i, t)| (i.into(), t))
    }

    /// Get the auxiliary data of the block.
    #[must_use]
    pub fn aux_data(&self) -> &BlockAuxData {
        &self.inner.aux_data
    }

    /// Returns a slot of the block.
    pub fn slot(&self) -> Slot {
        self.decode().slot().into()
    }
}

impl Display for MultiEraBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fork = self.fork;
        let block_data = &self.inner.data;
        let block = block_data.borrow_block();
        let block_number = block.number();
        let slot = block.slot();
        let size = block.size();
        let txns = block.tx_count();
        let aux_data = block.has_aux_data();

        let fork = if self.is_immutable() {
            "Immutable".to_string()
        } else {
            format!("Fork: {fork:?}")
        };

        let block_era = match block {
            pallas::ledger::traverse::MultiEraBlock::EpochBoundary(_) => {
                "Byron Epoch Boundary".to_string()
            },
            pallas::ledger::traverse::MultiEraBlock::AlonzoCompatible(_, era) => {
                format!("{era}")
            },
            pallas::ledger::traverse::MultiEraBlock::Babbage(_) => "Babbage".to_string(),
            pallas::ledger::traverse::MultiEraBlock::Byron(_) => "Byron".to_string(),
            pallas::ledger::traverse::MultiEraBlock::Conway(_) => "Conway".to_string(),
            _ => "Unknown".to_string(),
        };
        write!(f, "{block_era} block : {}, Previous {} : Slot# {slot} : {fork} : Block# {block_number} : Size {size} : Txns {txns} : AuxData? {aux_data}",
    self.point(), self.previous())?;
        Ok(())
    }
}

impl PartialEq for MultiEraBlock {
    /// Compare two `MultiEraBlock` by their current points.
    /// Ignores the Hash, we only check for equality of the Slot#.
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Eq for MultiEraBlock {}

impl PartialOrd for MultiEraBlock {
    /// Compare two `MultiEraBlock` by their points.
    /// Only checks the Slot#.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MultiEraBlock {
    /// Compare two `LiveBlocks` by their points.
    /// Only checks the Slot#.
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.point.cmp(&other.inner.point)
    }
}

// Allows us to compare a MultiEraBlock against a Point directly (Just the slot#).
impl PartialEq<Point> for MultiEraBlock {
    // Equality ONLY checks the Slot#
    fn eq(&self, other: &Point) -> bool {
        Some(Ordering::Equal) == self.partial_cmp(other)
    }
}

impl PartialOrd<Point> for MultiEraBlock {
    /// Compare a `MultiEraBlock` to a `Point` by their points.
    /// Only checks the Slot#.
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.inner.point.cmp(other))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Add;

    use anyhow::Ok;

    use crate::{multi_era_block_data::MultiEraBlock, network::Network, point::Point};

    struct TestRecord {
        raw: Vec<u8>,
        previous: Point,
    }

    /// Byron Test Block data
    fn byron_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/byron.block"))
            .expect("Failed to decode hex block.")
    }

    /// Shelley Test Block data
    fn shelley_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/shelley.block"))
            .expect("Failed to decode hex block.")
    }

    /// Mary Test Block data
    fn mary_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/mary.block")).expect("Failed to decode hex block.")
    }

    /// Allegra Test Block data
    fn allegra_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/allegra.block"))
            .expect("Failed to decode hex block.")
    }

    /// Alonzo Test Block data
    pub(crate) fn alonzo_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/allegra.block"))
            .expect("Failed to decode hex block.")
    }

    /// Babbage Test Block data
    pub(crate) fn babbage_block() -> Vec<u8> {
        hex::decode(include_str!("./../test_data/babbage.block"))
            .expect("Failed to decode hex block.")
    }

    /// An array of test blocks
    fn test_blocks() -> Vec<TestRecord> {
        vec![
            TestRecord {
                raw: byron_block(),
                previous: Point::ORIGIN,
            },
            TestRecord {
                raw: shelley_block(),
                previous: Point::ORIGIN,
            },
            TestRecord {
                raw: mary_block(),
                previous: Point::ORIGIN,
            },
            TestRecord {
                raw: allegra_block(),
                previous: Point::ORIGIN,
            },
            TestRecord {
                raw: alonzo_block(),
                previous: Point::ORIGIN,
            },
        ]
    }

    // Gets sorted by slot number from highest to lowest
    fn sorted_test_blocks() -> Vec<Vec<u8>> {
        vec![
            mary_block(),    // 27388606
            allegra_block(), // 18748707
            alonzo_block(),  // 18748707
            shelley_block(), // 7948610
            byron_block(),   // 3241381
        ]
    }

    /// Previous Point slot is >= blocks point, but hash is correct (should fail)
    #[test]
    fn test_multi_era_block_point_compare_1() -> anyhow::Result<()> {
        for (i, test_block) in test_blocks().into_iter().enumerate() {
            let pallas_block =
                pallas::ledger::traverse::MultiEraBlock::decode(test_block.raw.as_slice())?;

            let previous_point = Point::new(
                pallas_block.slot().add(i as u64).into(),
                pallas_block
                    .header()
                    .previous_hash()
                    .expect("cannot get previous hash")
                    .into(),
            );

            let block = MultiEraBlock::new(
                Network::Preprod,
                test_block.raw.clone(),
                &previous_point,
                1.into(),
            );

            assert!(block.is_err());
            assert!(block
                .unwrap_err()
                .to_string()
                .contains("Previous slot is not less than current slot"));
        }

        Ok(())
    }

    /// Previous Point slot is < blocks point, but hash is different. (should fail).
    #[test]
    fn test_multi_era_block_point_compare_2() -> anyhow::Result<()> {
        for test_block in test_blocks() {
            let pallas_block =
                pallas::ledger::traverse::MultiEraBlock::decode(test_block.raw.as_slice())?;

            let previous_point = Point::new(
                (pallas_block.slot().checked_sub(1).unwrap()).into(),
                vec![0; 32].try_into()?,
            );

            let block = MultiEraBlock::new(
                Network::Preprod,
                test_block.raw.clone(),
                &previous_point,
                1.into(),
            );

            assert!(block.is_err());
        }

        Ok(())
    }

    /// Previous Point slot is < blocks point, and hash is also correct. (should pass).
    #[test]
    fn test_multi_era_block_point_compare_3() -> anyhow::Result<()> {
        for test_block in test_blocks() {
            let pallas_block =
                pallas::ledger::traverse::MultiEraBlock::decode(test_block.raw.as_slice())?;

            let previous_point = Point::new(
                (pallas_block.slot().checked_sub(1).unwrap()).into(),
                pallas_block
                    .header()
                    .previous_hash()
                    .expect("cannot get previous hash")
                    .into(),
            );

            let block = MultiEraBlock::new(
                Network::Preprod,
                test_block.raw.clone(),
                &previous_point,
                1.into(),
            )?;

            assert_eq!(block.decode().hash(), pallas_block.hash());
        }

        Ok(())
    }

    fn mk_test_blocks() -> Vec<MultiEraBlock> {
        let raw_blocks = sorted_test_blocks();
        raw_blocks
            .iter()
            .map(|block| {
                let prev_point = pallas::ledger::traverse::MultiEraBlock::decode(block.as_slice())
                    .map(|block| {
                        Point::new(
                            (block.slot().saturating_sub(1)).into(),
                            block
                                .header()
                                .previous_hash()
                                .expect("cannot get previous hash")
                                .into(),
                        )
                    })
                    .expect("cannot create point");

                MultiEraBlock::new(Network::Preprod, block.clone(), &prev_point, 1.into())
                    .expect("cannot create multi-era block")
            })
            .collect()
    }

    fn mk_test_points() -> Vec<Point> {
        let raw_blocks = sorted_test_blocks();
        raw_blocks
            .iter()
            .map(|block| {
                pallas::ledger::traverse::MultiEraBlock::decode(block.as_slice())
                    .map(|block| {
                        Point::new(
                            block.slot().into(),
                            block
                                .header()
                                .previous_hash()
                                .expect("cannot get previous hash")
                                .into(),
                        )
                    })
                    .expect("cannot create point")
            })
            .collect()
    }

    /// Compares between blocks using comparison operators
    #[test]
    fn test_multi_era_block_point_compare_4() -> anyhow::Result<()> {
        let multi_era_blocks = mk_test_blocks();

        let mary_block = multi_era_blocks.first().expect("cannot get block");
        let allegra_block = multi_era_blocks.get(1).expect("cannot get block");
        let alonzo_block = multi_era_blocks.get(2).expect("cannot get block");
        let shelley_block = multi_era_blocks.get(3).expect("cannot get block");
        let byron_block = multi_era_blocks.get(4).expect("cannot get block");

        assert!(mary_block > allegra_block);
        assert!(mary_block >= allegra_block);
        assert!(mary_block != allegra_block);
        assert!(mary_block > alonzo_block);
        assert!(mary_block >= alonzo_block);
        assert!(mary_block != alonzo_block);
        assert!(mary_block > shelley_block);
        assert!(mary_block >= shelley_block);
        assert!(mary_block != shelley_block);
        assert!(mary_block > byron_block);
        assert!(mary_block >= byron_block);

        assert!(allegra_block < mary_block);
        assert!(allegra_block <= mary_block);
        assert!(allegra_block != mary_block);
        assert!(allegra_block == alonzo_block);
        assert!(allegra_block >= alonzo_block);
        assert!(allegra_block <= alonzo_block);
        assert!(allegra_block > shelley_block);
        assert!(allegra_block >= shelley_block);
        assert!(allegra_block != shelley_block);
        assert!(allegra_block > byron_block);
        assert!(allegra_block >= byron_block);
        assert!(allegra_block != byron_block);

        assert!(alonzo_block < mary_block);
        assert!(alonzo_block <= mary_block);
        assert!(alonzo_block != mary_block);
        assert!(alonzo_block == allegra_block);
        assert!(alonzo_block >= allegra_block);
        assert!(alonzo_block <= allegra_block);
        assert!(alonzo_block > shelley_block);
        assert!(alonzo_block >= shelley_block);
        assert!(alonzo_block != shelley_block);
        assert!(alonzo_block > byron_block);
        assert!(alonzo_block >= byron_block);
        assert!(alonzo_block != byron_block);

        assert!(shelley_block < mary_block);
        assert!(shelley_block <= mary_block);
        assert!(shelley_block != mary_block);
        assert!(shelley_block < allegra_block);
        assert!(shelley_block <= allegra_block);
        assert!(shelley_block != allegra_block);
        assert!(shelley_block < alonzo_block);
        assert!(shelley_block <= alonzo_block);
        assert!(shelley_block != alonzo_block);
        assert!(shelley_block > byron_block);
        assert!(shelley_block >= byron_block);
        assert!(shelley_block != byron_block);

        assert!(byron_block < mary_block);
        assert!(byron_block <= mary_block);
        assert!(byron_block != mary_block);
        assert!(byron_block < allegra_block);
        assert!(byron_block <= allegra_block);
        assert!(byron_block != allegra_block);
        assert!(byron_block < alonzo_block);
        assert!(byron_block <= alonzo_block);
        assert!(byron_block != alonzo_block);
        assert!(byron_block < shelley_block);
        assert!(byron_block <= shelley_block);
        assert!(byron_block != shelley_block);

        Ok(())
    }

    /// Compares between blocks and points using comparison operators
    #[test]
    fn test_multi_era_block_point_compare_5() -> anyhow::Result<()> {
        let points = mk_test_points();
        let blocks = mk_test_blocks();

        let mary_block = blocks.first().expect("cannot get block");
        let allegra_block = blocks.get(1).expect("cannot get block");
        let alonzo_block = blocks.get(2).expect("cannot get block");
        let shelley_block = blocks.get(3).expect("cannot get block");
        let byron_block = blocks.get(4).expect("cannot get block");

        let mary_point = points.first().expect("cannot get point");
        let allegra_point = points.get(1).expect("cannot get point");
        let alonzo_point = points.get(2).expect("cannot get point");
        let shelley_point = points.get(3).expect("cannot get point");
        let byron_point = points.get(4).expect("cannot get point");

        assert!(mary_block > allegra_point);
        assert!(mary_block >= allegra_point);
        assert!(mary_block != allegra_point);
        assert!(mary_block > alonzo_point);
        assert!(mary_block >= alonzo_point);
        assert!(mary_block != alonzo_point);
        assert!(mary_block > shelley_point);
        assert!(mary_block >= shelley_point);
        assert!(mary_block != shelley_point);
        assert!(mary_block > byron_point);
        assert!(mary_block >= byron_point);

        assert!(allegra_block < mary_point);
        assert!(allegra_block <= mary_point);
        assert!(allegra_block != mary_point);
        assert!(allegra_block == alonzo_point);
        assert!(allegra_block >= alonzo_point);
        assert!(allegra_block <= alonzo_point);
        assert!(allegra_block > shelley_point);
        assert!(allegra_block >= shelley_point);
        assert!(allegra_block != shelley_point);
        assert!(allegra_block > byron_point);
        assert!(allegra_block >= byron_point);
        assert!(allegra_block != byron_point);

        assert!(alonzo_block < mary_point);
        assert!(alonzo_block <= mary_point);
        assert!(alonzo_block != mary_point);
        assert!(alonzo_block == allegra_point);
        assert!(alonzo_block >= allegra_point);
        assert!(alonzo_block <= allegra_point);
        assert!(alonzo_block > shelley_point);
        assert!(alonzo_block >= shelley_point);
        assert!(alonzo_block != shelley_point);
        assert!(alonzo_block > byron_point);
        assert!(alonzo_block >= byron_point);
        assert!(alonzo_block != byron_point);

        assert!(shelley_block < mary_point);
        assert!(shelley_block <= mary_point);
        assert!(shelley_block != mary_point);
        assert!(shelley_block < allegra_point);
        assert!(shelley_block <= allegra_point);
        assert!(shelley_block != allegra_point);
        assert!(shelley_block < alonzo_point);
        assert!(shelley_block <= alonzo_point);
        assert!(shelley_block != alonzo_point);
        assert!(shelley_block > byron_point);
        assert!(shelley_block >= byron_point);
        assert!(shelley_block != byron_point);

        assert!(byron_block < mary_point);
        assert!(byron_block <= mary_point);
        assert!(byron_block != mary_point);
        assert!(byron_block < allegra_point);
        assert!(byron_block <= allegra_point);
        assert!(byron_block != allegra_point);
        assert!(byron_block < alonzo_point);
        assert!(byron_block <= alonzo_point);
        assert!(byron_block != alonzo_point);
        assert!(byron_block < shelley_point);
        assert!(byron_block <= shelley_point);
        assert!(byron_block != shelley_point);

        Ok(())
    }

    #[test]
    fn test_multi_era_block_with_origin_point() {
        for test_block in test_blocks() {
            let block = MultiEraBlock::new(
                Network::Preprod,
                test_block.raw.clone(),
                &test_block.previous,
                1.into(),
            );

            assert!(block.is_ok());
        }
    }
}
