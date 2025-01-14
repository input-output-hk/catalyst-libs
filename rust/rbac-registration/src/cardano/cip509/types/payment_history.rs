//! Payment history of the public key in tracking payment keys.

use pallas::{crypto::hash::Hash, ledger::primitives::conway::Value};

use super::point_tx_idx::PointTxIdx;

/// Payment history of the public key in tracking payment keys.
#[derive(Clone)]
pub struct PaymentHistory {
    /// The point and transaction index.
    point_tx_idx: PointTxIdx,
    /// Transaction hash that this payment come from.
    tx_hash: Hash<32>,
    /// The transaction output index that this payment come from.
    output_index: u16,
    /// The value of the payment.
    value: Value,
}

impl PaymentHistory {
    /// Create an instance of payment history.
    pub(crate) fn new(
        point_tx_idx: PointTxIdx, tx_hash: Hash<32>, output_index: u16, value: Value,
    ) -> Self {
        PaymentHistory {
            point_tx_idx,
            tx_hash,
            output_index,
            value,
        }
    }

    /// Get the point and transaction index.
    #[must_use]
    pub fn point_tx_idx(&self) -> &PointTxIdx {
        &self.point_tx_idx
    }

    /// Get the transaction hash.
    #[must_use]
    pub fn tx_hash(&self) -> Hash<32> {
        self.tx_hash
    }

    /// Get the transaction output index.
    #[must_use]
    pub fn output_index(&self) -> u16 {
        self.output_index
    }

    /// Get the value of the payment.
    #[must_use]
    pub fn value(&self) -> &Value {
        &self.value
    }
}
