//! Transaction Index
use crate::conversion::from_saturating;

/// Transaction index within a block
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L20C1-L20C33>
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TxnIndex(u16);

impl TxnIndex {
    /// Convert an `<T>` to transaction index (saturate if out of range).
    pub(crate) fn from_saturating<
        T: Copy
            + TryInto<u16>
            + std::ops::Sub<Output = T>
            + std::cmp::PartialOrd<T>
            + num_traits::identities::Zero,
    >(
        value: T,
    ) -> Self {
        let value: u16 = from_saturating(value);
        Self(value)
    }
}
