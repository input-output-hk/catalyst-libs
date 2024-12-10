//! Utility functions for CIP-19 address.

use anyhow::bail;

use crate::cardano::transaction::witness::TxWitness;

/// Extract the first 28 bytes from the given key
/// Refer to <https://cips.cardano.org/cip/CIP-19> for more information.
pub(crate) fn extract_key_hash(key: &[u8]) -> Option<Vec<u8>> {
    key.get(1..29).map(<[u8]>::to_vec)
}

/// Compare the given public key bytes with the transaction witness set.
pub(crate) fn compare_key_hash(
    pk_addrs: &[Vec<u8>], witness: &TxWitness, txn_idx: u16,
) -> anyhow::Result<()> {
    if pk_addrs.is_empty() {
        bail!("No public key addresses provided");
    }

    pk_addrs.iter().try_for_each(|pk_addr| {
        let pk_addr: [u8; 28] = pk_addr.as_slice().try_into().map_err(|_| {
            anyhow::anyhow!(
                "Invalid length for vkey, expected 28 bytes but got {}",
                pk_addr.len()
            )
        })?;

        // Key hash not found in the transaction witness set
        if !witness.check_witness_in_tx(&pk_addr, txn_idx) {
            bail!(
                "Public key hash not found in transaction witness set given {:?}",
                pk_addr
            );
        }

        Ok(())
    })
}
