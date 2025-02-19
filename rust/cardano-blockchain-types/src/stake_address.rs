//! A stake address.

use pallas::ledger::addresses::StakeAddress as PallasStakeAddress;

/// A stake address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct StakeAddress(PallasStakeAddress);

// TODO: FIXME
// impl StakeAddress {
//     pub fn new() -> Self {
//         todo!()
//     }
// }

impl From<PallasStakeAddress> for StakeAddress {
    fn from(value: PallasStakeAddress) -> Self {
        Self(value)
    }
}

/// This conversion returns a 29 bytes value that includes both header and hash.
impl From<StakeAddress> for Vec<u8> {
    fn from(value: StakeAddress) -> Self {
        // This `to_vec()` call includes both the header and the payload.
        value.0.to_vec()
    }
}
