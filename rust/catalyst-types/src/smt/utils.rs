//! Utility functions for the Sparse Merkle Tree module.

use sparse_merkle_tree::H256;

/// Maximum coarse height for the Sparse Merkle Tree, needed to limit the number of
/// batches.
const MAX_COARSE_HEIGHT: u8 = 14;

/// Calculates an appropriate coarse height for grouping Merkle tree nodes into batches.
///
/// This function determines a tree height that balances the tradeoff between batch size
/// and the number of batches when processing nodes. It's used to determine how to group
/// nodes at different levels of the tree for efficient batch processing.
///
/// Returns a height value between 1 and `MAX_COARSE_HEIGHT` that provides reasonable
/// batching for the given count.
pub(super) fn coarse_height(count: usize) -> u8 {
    let d = ((count as f64) / 64.0).max(1.0).log2();
    let d = d.max(1.0).min(f64::from(MAX_COARSE_HEIGHT));
    d.round() as u8
}

/// Generates a node key for a Sparse Merkle Tree based on the node's position in a
/// horizontal slice.
///
/// This function encodes a tree path into an H256 (32-byte array) by mapping the binary
/// representation of `position_from_left` into the most significant bits of the H256,
/// starting from bit 255 (the MSB) and working downward.
///
/// # Algorithm
///
/// 1. Takes the binary representation of `position_from_left`
/// 2. Extracts the first `key_prefix_length` bits (from MSB to LSB)
/// 3. Maps each bit to the H256 starting at bit 255:
///    - Position bit (`key_prefix_length` - 1) goes into H256 bit 255
///    - Position bit (`key_prefix_length` - 2) goes into H256 bit 254
///    - ...
///    - Position bit 0 goes into H256 bit (256 - `key_prefix_length`)
///
/// # Example
///
/// For `position_from_left = 26` and `key_prefix_length = 10`:
///
/// 1. Binary representation: `26 = 0b0000011010` (showing 10 bits)
///
/// 2. Initialize H256 with all zeros and then copy 10 bits going backward
///    - Position bit 9: `0` - Bit 255 - Do NOT set
///    - Position bit 8: `0` - Bit 254 - Do NOT set
///    - Position bit 7: `0` - Bit 253 - Do NOT set
///    - Position bit 6: `0` - Bit 252 - Do NOT set
///    - Position bit 5: `0` - Bit 251 - Do NOT set
///    - Position bit 4: `1` - Bit 250 - Set
///    - Position bit 3: `1` - Bit 249 - Set
///    - Position bit 2: `0` - Bit 248 - Do NOT set
///    - Position bit 1: `1` - Bit 247 - Set
///    - Position bit 0: `0` → Bit 246 - Do NOT set
///
/// 3. Bit-to-byte mapping:
///    - Bits 255-248 are in byte[31]: `0b00000110` = 6
///    - Bits 247-240 are in byte[30]: `0b10000000` = 128
///
/// 4. Result: `[0, 0, ..., 0, 128, 6]`
pub(super) fn node_key(
    key_prefix_length: u8,
    position_from_left: u32,
) -> H256 {
    let mut node_key = H256::zero();

    for bit_index_msb in 0..key_prefix_length {
        let bit_index_lsb = key_prefix_length - 1 - bit_index_msb;
        let mask = 1 << bit_index_lsb;
        let bit_is_set = (position_from_left & mask) != 0;

        if bit_is_set {
            node_key.set_bit(u8::MAX - bit_index_msb);
        }
    }
    node_key
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::smt::utils::{MAX_COARSE_HEIGHT, coarse_height, node_key};

    #[test_case(0 => 1)]
    #[test_case(1 => 1)]
    #[test_case(181 => 1)]
    #[test_case(182 => 2)]
    #[test_case(362 => 2)]
    #[test_case(363 => 3)]
    #[test_case(370_727 => 12)]
    #[test_case(370_728 => 13)]
    #[test_case(741_456 => MAX_COARSE_HEIGHT)]
    #[test_case(usize::MAX => MAX_COARSE_HEIGHT; "should be capped at 14")]
    fn calculates_coarse_height(count: usize) -> u8 {
        coarse_height(count)
    }

    #[test_case(1, 0 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]
    #[test_case(1, 1 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128])]
    #[test_case(2, 3 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192])]
    #[test_case(10, 26 => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 6])]
    fn creates_node_key(
        key_prefix_length: u8,
        horizontal_position: u32,
    ) -> [u8; 32] {
        let node_key = node_key(key_prefix_length, horizontal_position);
        node_key
            .as_slice()
            .try_into()
            .expect("should convert to slice")
    }
}
