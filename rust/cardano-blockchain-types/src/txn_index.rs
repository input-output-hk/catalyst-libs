//! Transaction Index
use catalyst_types::conversion::from_saturating;

/// Transaction index within a block
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L20C1-L20C33>
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxnIndex(u16);

impl<
        T: Copy
            + TryInto<u16>
            + std::ops::Sub<Output = T>
            + PartialOrd<T>
            + num_traits::identities::Zero,
    > From<T> for TxnIndex
{
    fn from(value: T) -> Self {
        Self(from_saturating(value))
    }
}

impl From<TxnIndex> for i16 {
    fn from(val: TxnIndex) -> Self {
        i16::try_from(val.0).unwrap_or(i16::MAX)
    }
}

impl From<TxnIndex> for usize {
    fn from(value: TxnIndex) -> Self {
        value.0.into()
    }
}

impl From<TxnIndex> for u16 {
    fn from(value: TxnIndex) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test for `From<T>` conversion to `TxnIndex`
    #[test]
    fn test_from_u8_to_txn_index() {
        let txn_index: TxnIndex = 100u8.into(); // u8 is a valid type for conversion
        assert_eq!(txn_index.0, 100);
    }

    #[test]
    fn test_from_u16_to_txn_index() {
        let txn_index: TxnIndex = 500u16.into(); // u16 is valid and within range for `TxnIndex`
        assert_eq!(txn_index.0, 500);
    }

    #[test]
    fn test_from_i32_to_txn_index() {
        let txn_index: TxnIndex = 1234i32.into(); // i32 can be converted into `TxnIndex`
        assert_eq!(txn_index.0, 1234);
    }

    #[test]
    fn test_from_u32_to_txn_index() {
        let txn_index: TxnIndex = 500_000u32.into(); // u32 is larger but should be saturated to `u16::MAX`
        assert_eq!(txn_index.0, u16::MAX);
    }

    #[test]
    fn test_from_large_i32_to_txn_index() {
        let txn_index: TxnIndex = 70000i32.into(); // i32 too large for u16, should saturate to `u16::MAX`
        assert_eq!(txn_index.0, u16::MAX);
    }

    // Test for `From<TxnIndex>` conversion to `i16`
    #[test]
    fn test_txn_index_to_i16_within_range() {
        let txn_index = TxnIndex(100);
        let result: i16 = txn_index.into(); // Should successfully convert to i16
        assert_eq!(result, 100);
    }

    #[test]
    fn test_txn_index_to_i16_with_saturation() {
        let txn_index = TxnIndex(u16::MAX); // u16::MAX = 65535, which is too large for i16
        let result: i16 = txn_index.into(); // Should saturate to i16::MAX
        assert_eq!(result, i16::MAX);
    }

    #[test]
    fn test_txn_index_to_i16_with_zero() {
        let txn_index = TxnIndex(0); // Should be able to convert to i16 without issue
        let result: i16 = txn_index.into();
        assert_eq!(result, 0);
    }
}
