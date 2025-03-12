//! Cardano hashing functions

use std::{fmt, str::FromStr};

use blake2b_simd::Params;
use displaydoc::Display;
use pallas_crypto::hash::Hash;
use thiserror::Error;

/// Number of bytes in a blake2b 224 hash.
pub const BLAKE_2B224_SIZE: usize = 224 / 8;

/// `Blake2B` 224bit Hash.
pub type Blake2b224Hash = Blake2bHash<BLAKE_2B224_SIZE>;

/// Number of bytes in a blake2b 256 hash.
pub const BLAKE_2B256_SIZE: usize = 256 / 8;

/// `Blake2B` 256bit Hash
pub type Blake2b256Hash = Blake2bHash<BLAKE_2B256_SIZE>;

/// Number of bytes in a blake2b 128 hash.
pub const BLAKE_2B128_SIZE: usize = 128 / 8;

/// `Blake2B` 128bit Hash
pub type Blake2b128Hash = Blake2bHash<BLAKE_2B128_SIZE>;

/// Errors that can occur when converting to a `Blake2bHash`.
#[derive(Display, Debug, Error)]
pub enum Blake2bHashError {
    /// Invalid length: expected {expected} bytes, got {actual}
    InvalidLength {
        /// The expected number of bytes (must be 32 or 28).
        expected: usize,
        /// The actual number of bytes in the provided input.
        actual: usize,
    },
    /// Invalid hex string: {source}
    InvalidHex {
        /// The underlying error from `hex`.
        #[from]
        source: hex::FromHexError,
    },
}

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

impl<const BYTES: usize> From<Blake2bHash<BYTES>> for Hash<BYTES> {
    #[inline]
    fn from(hash: Blake2bHash<BYTES>) -> Self {
        hash.0
    }
}

impl<const BYTES: usize> From<Blake2bHash<BYTES>> for Vec<u8> {
    fn from(val: Blake2bHash<BYTES>) -> Self {
        val.0.to_vec()
    }
}

/// Convert hash in a form of byte array into the `Blake2bHash` type.
impl<const BYTES: usize> TryFrom<&[u8]> for Blake2bHash<BYTES> {
    type Error = Blake2bHashError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < BYTES {
            return Err(Blake2bHashError::InvalidLength {
                expected: BYTES,
                actual: value.len(),
            });
        }

        let mut hash = [0; BYTES];
        hash.copy_from_slice(value);
        let hash: Hash<BYTES> = hash.into();
        Ok(hash.into())
    }
}

impl<const BYTES: usize> TryFrom<&Vec<u8>> for Blake2bHash<BYTES> {
    type Error = Blake2bHashError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl<const BYTES: usize> TryFrom<Vec<u8>> for Blake2bHash<BYTES> {
    type Error = Blake2bHashError;

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
    type Err = Blake2bHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash: Hash<BYTES> = s.parse().map_err(Blake2bHashError::from)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b_hash_init() {
        let data = b"Cardano";
        let hash_224 = Blake2b224Hash::new(data);
        let hash_256 = Blake2b256Hash::new(data);
        let hash_128 = Blake2b128Hash::new(data);

        assert_eq!(hash_224.0.as_ref().len(), BLAKE_2B224_SIZE);
        assert_eq!(hash_256.0.as_ref().len(), BLAKE_2B256_SIZE);
        assert_eq!(hash_128.0.as_ref().len(), BLAKE_2B128_SIZE);
    }

    #[test]
    fn test_blake2b_hash_conversion() {
        let data = b"Cardano";
        let hash = Blake2b224Hash::new(data);

        let as_vec: Vec<u8> = hash.into();
        let from_vec = Blake2b224Hash::try_from(&as_vec).unwrap();
        assert_eq!(hash, from_vec);

        let from_slice = Blake2b224Hash::try_from(as_vec.as_slice()).unwrap();
        assert_eq!(hash, from_slice);
    }

    #[test]
    fn test_blake2b_hash_invalid_length() {
        let invalid_data = vec![0u8; 10];
        let result = Blake2b224Hash::try_from(&invalid_data);
        assert!(result.is_err());

        if let Err(Blake2bHashError::InvalidLength { expected, actual }) = result {
            assert_eq!(expected, BLAKE_2B224_SIZE);
            assert_eq!(actual, invalid_data.len());
        } else {
            panic!("Expected InvalidLength error");
        }
    }
}
