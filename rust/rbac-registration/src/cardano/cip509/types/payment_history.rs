//! Payment history of the public key in tracking payment keys.

use std::collections::HashMap;

use cardano_blockchain_types::{
    pallas_addresses::ShelleyAddress,
    pallas_primitives::{Hash, conway::Value},
};

use super::point_tx_idx::PointTxnIdx;

/// A map from address to a list of payments.
pub type PaymentHistory = HashMap<ShelleyAddress, Vec<Payment>>;

/// Payment history of the public key in tracking payment keys.
#[derive(Debug, Clone)]
pub struct Payment {
    /// The point and transaction index.
    point_tx_idx: PointTxnIdx,
    /// Transaction hash that this payment come from.
    tx_hash: Hash<32>,
    /// The transaction output index that this payment come from.
    output_index: u16,
    /// The value of the payment.
    value: Value,
}

impl Payment {
    /// Create an instance of payment history.
    pub(crate) fn new(
        point_tx_idx: PointTxnIdx,
        tx_hash: Hash<32>,
        output_index: u16,
        value: Value,
    ) -> Self {
        Payment {
            point_tx_idx,
            tx_hash,
            output_index,
            value,
        }
    }

    /// Get the point and transaction index.
    #[must_use]
    pub fn point_tx_idx(&self) -> &PointTxnIdx {
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
