//! Transaction input hash type

use anyhow::Context;
use cardano_blockchain_types::hashes::Blake2b128Hash;

/// A 16-byte hash of the transaction inputs field.
///
/// This type is described [here].
///
/// [here]: https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md#key-1-txn-inputs-hash
#[derive(Debug, PartialEq, Clone)]
pub struct TxInputHash(Blake2b128Hash);

impl From<[u8; 16]> for TxInputHash {
    fn from(bytes: [u8; 16]) -> Self {
        Self(Blake2b128Hash::from(bytes))
    }
}

impl TryFrom<&[u8]> for TxInputHash {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let hash = Blake2b128Hash::try_from(value).context("Invalid transaction input hash")?;
        Ok(Self(hash))
    }
}

impl From<Blake2b128Hash> for TxInputHash {
    fn from(value: Blake2b128Hash) -> Self {
        Self(value)
    }
}
