//! Utility functions for ZK unit vector proofs.

/// Calculates the `bit_index`-th bit of the `val`.
/// `val` is represented in little-endian format.
#[inline]
pub(crate) fn get_bit(val: u8, bit_index: usize) -> bool {
    ((val.to_le() >> bit_index) & 1) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bit_test() {
        assert!(!get_bit(0b101_000, 0));
        assert!(get_bit(0b101_010, 1));
        assert!(!get_bit(0b101_010, 2));
        assert!(get_bit(0b101_010, 3));
        assert!(!get_bit(0b101_010, 4));
        assert!(get_bit(0b101_010, 5));
    }
}
