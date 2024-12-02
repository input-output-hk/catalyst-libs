//! Utility functions for CIP-19 address.

use anyhow::bail;
use regex::Regex;

use crate::cardano::transaction::witness::TxWitness;

/// Extracts the CIP-19 bytes from a URI.
/// Example input: `web+cardano://addr/<cip-19 address string>`
/// <https://github.com/cardano-foundation/CIPs/tree/6bae5165dde5d803778efa5e93bd408f3317ca03/CPS-0016>
/// URI = scheme ":" ["//" authority] path ["?" query] ["#" fragment]
#[must_use]
pub fn extract_cip19_hash(uri: &str, prefix: Option<&str>) -> Option<Vec<u8>> {
    // Regex pattern to match the expected URI format
    let r = Regex::new("^.+://addr/(.+)$").ok()?;

    // Apply the regex pattern to capture the CIP-19 address string
    let address = r
        .captures(uri)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));

    match address {
        Some(addr) => {
            if let Some(prefix) = prefix {
                if !addr.starts_with(prefix) {
                    return None;
                }
            }
            let addr = bech32::decode(&addr).ok()?.1;
            // As in CIP19, the first byte is the header, so extract only the payload
            extract_key_hash(&addr)
        },
        None => None,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // Test data from https://cips.cardano.org/cip/CIP-19
    // cSpell:disable
    const STAKE_ADDR: &str = "stake_test1uqehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gssrtvn";
    const PAYMENT_ADDR: &str = "addr_test1qz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgs68faae";
    // cSpell:enable

    #[test]
    fn test_extract_cip19_hash_with_stake() {
        // Additional tools to check for bech32 https://slowli.github.io/bech32-buffer/
        let uri = &format!("web+cardano://addr/{STAKE_ADDR}");
        // Given:
        // e0337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251
        // The first byte is the header, so extract only the payload
        let bytes = hex::decode("337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251")
            .expect("Failed to decode bytes");
        assert_eq!(
            extract_cip19_hash(uri, Some("stake")).expect("Failed to extract CIP-19 hash"),
            bytes
        );
    }

    #[test]
    fn test_extract_cip19_hash_with_addr_with_prefix_set() {
        let uri = &format!("web+cardano://addr/{PAYMENT_ADDR}");
        let result = extract_cip19_hash(uri, Some("stake"));
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cip19_hash_with_addr_without_prefix_set() {
        let uri = &format!("web+cardano://addr/{PAYMENT_ADDR}");
        let result = extract_cip19_hash(uri, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_cip19_hash_invalid_uri() {
        let uri = "invalid_uri";
        let result = extract_cip19_hash(uri, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cip19_hash_non_bech32_address() {
        let uri = "example://addr/not_bech32";
        let result = extract_cip19_hash(uri, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cip19_hash_empty_uri() {
        let uri = "";
        let result = extract_cip19_hash(uri, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cip19_hash_no_address() {
        let uri = "example://addr/";
        let result = extract_cip19_hash(uri, None);
        assert_eq!(result, None);
    }
}
