//! Utility functions for CIP-19 address.

use anyhow::bail;
use cardano_blockchain_types::{TxnIndex, TxnWitness, VKeyHash};

/// Extract the first 28 bytes from the given key
/// Refer to <https://cips.cardano.org/cip/CIP-19> for more information.
pub(crate) fn extract_key_hash(key: &[u8]) -> Option<VKeyHash> {
    key.get(1..29).and_then(|v| v.try_into().ok())
}

/// Compare the given public key bytes with the transaction witness set.
pub(crate) fn compare_key_hash(
    pk_addrs: &[VKeyHash], witness: &TxnWitness, txn_idx: TxnIndex,
) -> anyhow::Result<()> {
    if pk_addrs.is_empty() {
        bail!("No public key addresses provided");
    }

    pk_addrs.iter().try_for_each(|pk_addr| {
        // Key hash not found in the transaction witness set
        if !witness.check_witness_in_tx(pk_addr, txn_idx) {
            bail!("Public key hash not found in transaction witness set given {pk_addr}",);
        }

        Ok(())
    })
}
