//! Utility functions for CIP-19 address.

use cardano_blockchain_types::VKeyHash;

/// Extract the first 28 bytes from the given key
/// Refer to <https://cips.cardano.org/cip/CIP-19> for more information.
pub(crate) fn extract_key_hash(key: &[u8]) -> Option<VKeyHash> {
    key.get(1..29).and_then(|v| v.try_into().ok())
}
