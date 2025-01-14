//! Point or absolute slot and transaction index.

use pallas::network::miniprotocols::Point;

/// Point (slot) and transaction index.
#[derive(Debug, Clone)]
pub struct PointTxIdx((Point, usize));

impl PointTxIdx {
    /// Create an instance of point and transaction index.
    pub(crate) fn new(point: Point, tx_idx: usize) -> Self {
        PointTxIdx((point, tx_idx))
    }

    /// Get the point.
    #[must_use]
    pub fn point(&self) -> &Point {
        &self.0 .0
    }

    /// Get the transaction index.
    #[must_use]
    pub fn tx_idx(&self) -> usize {
        self.0 .1
    }
}
