//! A transaction output offset inside the transaction.
use catalyst_types::conversion::from_saturating;

/// A transaction output offset inside the transaction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxnOutputOffset(u16);

impl<
        T: Copy
            + TryInto<u16>
            + std::ops::Sub<Output = T>
            + PartialOrd<T>
            + num_traits::identities::Zero,
    > From<T> for TxnOutputOffset
{
    fn from(value: T) -> Self {
        Self(from_saturating(value))
    }
}

impl From<TxnOutputOffset> for i16 {
    fn from(val: TxnOutputOffset) -> Self {
        i16::try_from(val.0).unwrap_or(i16::MAX)
    }
}

impl From<TxnOutputOffset> for usize {
    fn from(value: TxnOutputOffset) -> Self {
        value.0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u8_to_txn_index() {
        let txn_index: TxnOutputOffset = 100u8.into(); // u8 is a valid type for conversion
        assert_eq!(txn_index.0, 100);
    }

    #[test]
    fn test_from_u16_to_txn_index() {
        let txn_index: TxnOutputOffset = 500u16.into(); // u16 is valid and within range for `TxnOutputOffset`
        assert_eq!(txn_index.0, 500);
    }

    #[test]
    fn test_from_i32_to_txn_index() {
        let txn_index: TxnOutputOffset = 1234i32.into(); // i32 can be converted into `TxnOutputOffset`
        assert_eq!(txn_index.0, 1234);
    }

    #[test]
    fn test_from_u32_to_txn_index() {
        let txn_index: TxnOutputOffset = 500_000u32.into(); // u32 is larger but should be saturated to `u16::MAX`
        assert_eq!(txn_index.0, u16::MAX);
    }

    #[test]
    fn test_from_large_i32_to_txn_index() {
        let txn_index: TxnOutputOffset = 70000i32.into(); // i32 too large for u16, should saturate to `u16::MAX`
        assert_eq!(txn_index.0, u16::MAX);
    }

    #[test]
    fn test_txn_index_to_i16_within_range() {
        let txn_index = TxnOutputOffset(100);
        let result: i16 = txn_index.into(); // Should successfully convert to i16
        assert_eq!(result, 100);
    }

    #[test]
    fn test_txn_index_to_i16_with_saturation() {
        let txn_index = TxnOutputOffset(u16::MAX); // u16::MAX = 65535, which is too large for i16
        let result: i16 = txn_index.into(); // Should saturate to i16::MAX
        assert_eq!(result, i16::MAX);
    }

    #[test]
    fn test_txn_index_to_i16_with_zero() {
        let txn_index = TxnOutputOffset(0); // Should be able to convert to i16 without issue
        let result: i16 = txn_index.into();
        assert_eq!(result, 0);
    }
}
