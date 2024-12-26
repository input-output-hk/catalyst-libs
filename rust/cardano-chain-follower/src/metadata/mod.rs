//! Metadata decoding and validating.

use std::{fmt::Debug, sync::Arc};

use cardano_blockchain_types::{MetadatumLabel, Network, TransactionAuxData};
use cip36::Cip36;
use cip509::Cip509;
use dashmap::DashMap;
use pallas::ledger::traverse::MultiEraTx;

pub mod cip36;
pub mod cip509;

/// List of all validation errors (as strings) Metadata is considered Valid if this list
/// is empty.
pub type ValidationReport = Vec<String>;

/// Possible Decoded Metadata Values.
/// Must match the key they relate too, but the consumer needs to check this.
#[derive(Debug)]
pub enum DecodedMetadataValues {
    // Json Metadata // TODO
    // Json(serde_json::Value), // TODO
    /// CIP-36/CIP-15 Catalyst Registration metadata.
    Cip36(Arc<Cip36>),
    /// CIP-509 RBAC metadata.
    Cip509(Arc<Cip509>),
}

/// An individual decoded metadata item.
#[derive(Debug)]
pub struct DecodedMetadataItem {
    /// The decoded metadata itself.
    pub value: DecodedMetadataValues,
    /// Validation report for this metadata item.
    pub report: ValidationReport,
}

/// Decoded Metadata for a single transaction.
/// The key is the Primary Label of the Metadata.  
/// For example, CIP15/36 uses labels 61284 & 61285,
/// 61284 is the primary label, so decoded metadata
/// will be under that label.
pub(crate) struct DecodedMetadata(DashMap<MetadatumLabel, Arc<DecodedMetadataItem>>);

impl DecodedMetadata {
    /// Create new decoded metadata for a transaction.
    fn new(chain: Network, slot: u64, txn: &MultiEraTx, raw_aux_data: &TransactionAuxData) -> Self {
        let decoded_metadata = Self(DashMap::new());

        // Process each known type of metadata here, and record the decoded result.
        Cip36::decode_and_validate(&decoded_metadata, slot, txn, raw_aux_data, true, chain);
        Cip509::decode_and_validate(&decoded_metadata, txn, raw_aux_data);

        // if !decoded_metadata.0.is_empty() {
        //    debug!("Decoded Metadata final: {decoded_metadata:?}");
        //}
        decoded_metadata
    }

    /// Get the decoded metadata item at the given slot, or None if it doesn't exist.
    pub fn get(&self, primary_label: u64) -> Option<Arc<DecodedMetadataItem>> {
        let entry = self.0.get(&primary_label)?;
        let value = entry.value();
        Some(value.clone())
    }
}

impl Debug for DecodedMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecodedMetadata {")?;
        for kv in &self.0 {
            let k = kv.key();
            let v = kv.value().clone();
            f.write_fmt(format_args!("{k:?}:{v:?} "))?;
        }
        f.write_str("}")
    }
}
