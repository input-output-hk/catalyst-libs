//! Decoded Metadata for a Block

use std::sync::Arc;

use anyhow::bail;
use dashmap::DashMap;
use pallas::ledger::traverse::MultiEraBlock;

use super::aux_data::TransactionAuxData;
use crate::txn_index::TxnIndex;

/// Auxiliary Data for every transaction within a block.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct BlockAuxData(Arc<dashmap::ReadOnlyView<TxnIndex, TransactionAuxData>>);

impl BlockAuxData {
    /// Get `TransactionAuxData` for the given `TxnIndex` if any.
    #[must_use]
    pub fn get(&self, txn_idx: TxnIndex) -> Option<&TransactionAuxData> {
        self.0.get(&txn_idx)
    }
}

impl Default for BlockAuxData {
    fn default() -> Self {
        BlockAuxData(Arc::new(DashMap::default().into_read_only()))
    }
}

impl TryFrom<&MultiEraBlock<'_>> for BlockAuxData {
    type Error = anyhow::Error;

    fn try_from(block: &MultiEraBlock) -> Result<Self, Self::Error> {
        let aux_data = DashMap::<TxnIndex, TransactionAuxData>::new();
        // Note, while this code looks redundant, it is not because all the types are not
        // compatible Even though they have similar names, and ultimately the same inner
        // functionality. This means we need to distinctly encode the three different
        // loops with the same code.
        if block.has_aux_data() {
            if let Some(_metadata) = block.as_byron() {
                // Nothing to do here.
            } else if let Some(alonzo_block) = block.as_alonzo() {
                for (txn_idx, metadata) in alonzo_block.auxiliary_data_set.iter() {
                    let mut d = minicbor::Decoder::new(metadata.raw_cbor());
                    let txn_aux_data = d.decode::<TransactionAuxData>()?;
                    aux_data.insert(TxnIndex::from_saturating(*txn_idx), txn_aux_data);
                }
            } else if let Some(babbage_block) = block.as_babbage() {
                for (txn_idx, metadata) in babbage_block.auxiliary_data_set.iter() {
                    let mut d = minicbor::Decoder::new(metadata.raw_cbor());
                    let txn_aux_data = d.decode::<TransactionAuxData>()?;
                    aux_data.insert(TxnIndex::from_saturating(*txn_idx), txn_aux_data);
                }
            } else if let Some(conway_block) = block.as_conway() {
                for (txn_idx, metadata) in conway_block.auxiliary_data_set.iter() {
                    let mut d = minicbor::Decoder::new(metadata.raw_cbor());
                    let txn_aux_data = d.decode::<TransactionAuxData>()?;
                    aux_data.insert(TxnIndex::from_saturating(*txn_idx), txn_aux_data);
                }
            } else {
                bail!("Undecodable metadata, unknown Era");
            };
        }

        Ok(Self(Arc::new(aux_data.into_read_only())))
    }
}
