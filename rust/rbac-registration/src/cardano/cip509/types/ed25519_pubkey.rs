//! Ed25519 public key type.

use pallas::codec::utils::Bytes;

/// 32 bytes Ed25519 public key.
#[derive(Debug, PartialEq, Clone, Default, Eq, Hash)]
pub struct Ed25519PublicKey([u8; 32]);

impl From<[u8; 32]> for Ed25519PublicKey {
    fn from(bytes: [u8; 32]) -> Self {
        Ed25519PublicKey(bytes)
    }
}

impl TryFrom<Bytes> for Ed25519PublicKey {
    type Error = &'static str;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        let byte_vec: Vec<u8> = bytes.into();

        if byte_vec.len() != 32 {
            return Err("Invalid length for Ed25519 public key: expected 32 bytes.");
        }

        let byte_array: [u8; 32] = byte_vec
            .try_into()
            .map_err(|_| "Failed to convert Vec<u8> to [u8; 32]")?;

        Ok(Ed25519PublicKey::from(byte_array))
    }
}

impl From<Ed25519PublicKey> for Bytes {
    fn from(val: Ed25519PublicKey) -> Self {
        let vec: Vec<u8> = val.0.to_vec();
        Bytes::from(vec)
    }
}

impl From<Ed25519PublicKey> for Vec<u8> {
    fn from(val: Ed25519PublicKey) -> Self {
        val.0.to_vec()
    }
}

impl From<Ed25519PublicKey> for [u8; 32] {
    fn from(val: Ed25519PublicKey) -> Self {
        val.0
    }
}
