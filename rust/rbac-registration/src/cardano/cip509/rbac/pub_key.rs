//! Public key type for RBAC metadata

use minicbor::{decode, Decode, Decoder};
use pallas::codec::utils::Bytes;

use super::tag::KeyTag;
use crate::utils::decode_helper::{decode_bytes, decode_tag};

/// Enum of possible public key type.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum SimplePublicKeyType {
    /// Undefined indicates skipped element.
    #[default]
    Undefined,
    /// Deleted indicates the key is deleted.
    Deleted,
    /// Ed25519 public key.
    Ed25519(Ed25519PublicKey),
}

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

impl Decode<'_, ()> for SimplePublicKeyType {
    fn decode(d: &mut Decoder, _ctx: &mut ()) -> Result<Self, decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Tag => {
                let tag = decode_tag(d, "SimplePublicKeyType")?;
                match tag {
                    t if t == KeyTag::Deleted.tag() => Ok(Self::Deleted),
                    t if t == KeyTag::Ed25519.tag() => {
                        let bytes = decode_bytes(d, "Ed25519 SimplePublicKeyType")?;
                        let mut ed25519 = [0u8; 32];
                        if bytes.len() == 32 {
                            ed25519.copy_from_slice(&bytes);
                            Ok(Self::Ed25519(Ed25519PublicKey(ed25519)))
                        } else {
                            Err(decode::Error::message(format!(
                                "Invalid length for Ed25519 key, got {}",
                                bytes.len()
                            )))
                        }
                    },
                    _ => Err(decode::Error::message("Unknown tag for Self")),
                }
            },
            minicbor::data::Type::Undefined => Ok(Self::Undefined),
            _ => Err(decode::Error::message(
                "Invalid datatype for SimplePublicKeyType",
            )),
        }
    }
}
