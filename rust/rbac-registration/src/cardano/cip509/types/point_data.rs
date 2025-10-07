//! Point and transaction index with its associated data.

use cardano_blockchain_types::{Point, TxnIndex};

use super::PointTxnIdx;

/// Point + transaction index with data.
#[derive(Debug, Clone, PartialEq)]
pub struct PointData<T> {
    /// Point and transaction index.
    point_txn_index: PointTxnIdx,
    /// Data associated to the point and transaction index.
    data: T,
}

impl<T> PointData<T> {
    /// Creates an instance of point and transaction index with data.
    #[must_use]
    pub fn new(
        point_txn_index: PointTxnIdx,
        data: T,
    ) -> Self {
        Self {
            point_txn_index,
            data,
        }
    }

    /// Get a reference to the data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get the point.
    pub fn point(&self) -> &Point {
        self.point_txn_index.point()
    }

    /// Get the transaction index.
    pub fn txn_index(&self) -> TxnIndex {
        self.point_txn_index.txn_index()
    }
}
