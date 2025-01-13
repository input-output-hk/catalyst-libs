//! A local key reference.

use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use crate::{cardano::cip509::rbac::Cip509RbacMetadataInt, utils::decode_helper::decode_helper};

/// Local key reference.
#[derive(Debug, PartialEq, Clone)]
pub struct KeyLocalRef {
    /// Local reference.
    pub local_ref: LocalRefInt,
    /// Key offset.
    pub key_offset: u64,
}

/// Enum of local reference with its associated unsigned integer value.
#[derive(FromRepr, Debug, PartialEq, Clone, Eq, Hash)]
#[repr(u8)]
pub enum LocalRefInt {
    /// x509 certificates.
    X509Certs = Cip509RbacMetadataInt::X509Certs as u8, // 10
    /// c509 certificates.
    C509Certs = Cip509RbacMetadataInt::C509Certs as u8, // 20
    /// Public keys.
    PubKeys = Cip509RbacMetadataInt::PubKeys as u8, // 30
}

impl Decode<'_, ()> for KeyLocalRef {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let local_ref = LocalRefInt::from_repr(decode_helper(d, "LocalRef in KeyLocalRef", ctx)?)
            .ok_or(decode::Error::message("Invalid local reference"))?;
        let key_offset: u64 = decode_helper(d, "KeyOffset in KeyLocalRef", ctx)?;
        Ok(Self {
            local_ref,
            key_offset,
        })
    }
}
