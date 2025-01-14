//! Point or absolute slot and transaction index.

use cardano_blockchain_types::{MultiEraBlock, Point, TxnIndex};

/// Point (slot) and transaction index.
#[derive(Debug, Clone, PartialEq)]
pub struct PointTxnIdx {
    point: Point,
    txn_index: TxnIndex,
}

impl PointTxnIdx {
    /// Creates an instance of point and transaction index.
    pub fn new(point: Point, txn_index: TxnIndex) -> Self {
        Self { point, txn_index }
    }

    /// Creates an instance of `PointTxnIdx` from the given block and index.
    pub fn from_block(block: &MultiEraBlock, txn_index: TxnIndex) -> Self {
        Self::new(block.point(), txn_index)
    }

    /// Get the point.
    #[must_use]
    pub fn point(&self) -> &Point {
        &self.point
    }

    /// Get the transaction index.
    #[must_use]
    pub fn txn_index(&self) -> TxnIndex {
        self.txn_index
    }
}
