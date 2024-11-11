//! Public key type for RBAC metadata

use minicbor::{decode, Decode, Decoder};

use crate::utils::decode_helper::{decode_bytes, decode_tag};

use super::tag::KeyTag;

/// Enum of possible public key type.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum SimplePublicKeyType {
    /// Undefined indicates skipped element.
    #[default]
    Undefined,
    /// Deleted indicates the key is deleted.
    Deleted,
    /// Ed25519 key.
    Ed25519([u8; 32]),
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
                            Ok(Self::Ed25519(ed25519))
                        } else {
                            Err(decode::Error::message("Invalid length for Ed25519 key"))
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
