//! Cardano hashing functions

use std::{fmt, str::FromStr};

use anyhow::bail;
use blake2b_simd::Params;
use pallas_crypto::hash::Hash;

/// Number of bytes in a blake2b 224 hash.
pub const BLAKE_2B224_SIZE: usize = 224 / 8;
/// `Blake2B` 224bit Hash
pub type Blake2b224Hash = Blake2bHash<BLAKE_2B224_SIZE>;

/// Number of bytes in a blake2b 256 hash.
pub const BLAKE_2B256_SIZE: usize = 256 / 8;
/// `Blake2B` 256bit Hash
pub type Blake2b256Hash = Blake2bHash<BLAKE_2B256_SIZE>;

/// Number of bytes in a blake2b 128 hash.
pub const BLAKE_2B128_SIZE: usize = 128 / 8;
/// `Blake2B` 128bit Hash
pub type Blake2b128Hash = Blake2bHash<BLAKE_2B128_SIZE>;

/// data that is a blake2b [`struct@Hash`] of `BYTES` long.
///
/// Possible values with Cardano are 32 bytes long (block hash or transaction
/// hash). Or 28 bytes long (as used in addresses)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake2bHash<const BYTES: usize>(Hash<BYTES>);

impl<const BYTES: usize> Blake2bHash<BYTES> {
    /// Create a new `Blake2bHash` from a slice of bytes by hashing them.
    #[must_use]
    pub fn new(input_bytes: &[u8]) -> Self {
        let mut bytes: [u8; BYTES] = [0u8; BYTES];

        // Generate a unique hash of the data.
        let mut hasher = Params::new().hash_length(BYTES).to_state();

        hasher.update(input_bytes);
        let hash = hasher.finalize();

        // Create a new array containing the first BYTES elements from the original array
        bytes.copy_from_slice(hash.as_bytes());

        bytes.into()
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for Blake2bHash<BYTES> {
    #[inline]
    fn from(bytes: [u8; BYTES]) -> Self {
        let hash: Hash<BYTES> = bytes.into();
        hash.into()
    }
}

impl<const BYTES: usize> From<Hash<BYTES>> for Blake2bHash<BYTES> {
    #[inline]
    fn from(bytes: Hash<BYTES>) -> Self {
        Self(bytes)
    }
}

impl<const BYTES: usize> From<Blake2bHash<BYTES>> for Vec<u8> {
    fn from(val: Blake2bHash<BYTES>) -> Self {
        val.0.to_vec()
    }
}

/// Convert hash in a form of byte array into the `Blake2bHash` type.
impl<const BYTES: usize> TryFrom<&[u8]> for Blake2bHash<BYTES> {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < BYTES {
            bail!("Invalid input length");
        }

        let mut hash = [0; BYTES];
        hash.copy_from_slice(value);
        let hash: Hash<BYTES> = hash.into();
        Ok(hash.into())
    }
}

impl<const BYTES: usize> TryFrom<&Vec<u8>> for Blake2bHash<BYTES> {
    type Error = anyhow::Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl<const BYTES: usize> TryFrom<Vec<u8>> for Blake2bHash<BYTES> {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl<const BYTES: usize> fmt::Debug for Blake2bHash<BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:?}", self.0))
    }
}

impl<const BYTES: usize> fmt::Display for Blake2bHash<BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

impl<const BYTES: usize> FromStr for Blake2bHash<BYTES> {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash: Hash<BYTES> = s.parse()?;
        Ok(hash.into())
    }
}

impl<C, const BYTES: usize> minicbor::Encode<C> for Blake2bHash<BYTES> {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(self.0.as_ref())?.ok()
    }
}

impl<'a, C, const BYTES: usize> minicbor::Decode<'a, C> for Blake2bHash<BYTES> {
    fn decode(
        d: &mut minicbor::Decoder<'a>, _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;
        bytes.try_into().map_err(|_| {
            minicbor::decode::Error::message("Invalid hash size for Blake2bHash cbor decode")
        })
    }
}
