//! Role data for RBAC metadata.

use std::collections::HashMap;

use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use super::{decode_any, decode_map_len, Cip509RbacMetadataInt};
use crate::utils::decode_helper::{decode_array_len, decode_helper};

/// Role data.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct RoleData {
    /// Role number.
    pub role_number: u8,
    /// Optional role signing key.
    pub role_signing_key: Option<KeyLocalRef>,
    /// Optional role encryption key.
    pub role_encryption_key: Option<KeyLocalRef>,
    /// Optional payment key.
    pub payment_key: Option<i16>,
    /// Optional role extended data keys.
    /// Empty map if no role extended data keys.
    pub role_extended_data_keys: HashMap<u8, Vec<u8>>,
}

/// The first valid role extended data key.
const FIRST_ROLE_EXT_KEY: u8 = 10;
/// The last valid role extended data key.
const LAST_ROLE_EXT_KEY: u8 = 99;

/// Enum of role data with its associated unsigned integer value.
#[allow(clippy::module_name_repetitions)]
#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub enum RoleDataInt {
    /// Role number.
    RoleNumber = 0,
    /// Role signing key.
    RoleSigningKey = 1,
    /// Role encryption key.
    RoleEncryptionKey = 2,
    /// Payment key.
    PaymentKey = 3,
}

impl Decode<'_, ()> for RoleData {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "RoleData")?;
        let mut role_data = RoleData::default();
        for _ in 0..map_len {
            let key: u8 = decode_helper(d, "key in RoleData", ctx)?;
            if let Some(key) = RoleDataInt::from_repr(key) {
                match key {
                    RoleDataInt::RoleNumber => {
                        role_data.role_number = decode_helper(d, "RoleNumber in RoleData", ctx)?;
                    },
                    RoleDataInt::RoleSigningKey => {
                        decode_array_len(d, "RoleSigningKey")?;
                        role_data.role_signing_key = Some(KeyLocalRef::decode(d, ctx)?);
                    },
                    RoleDataInt::RoleEncryptionKey => {
                        decode_array_len(d, "RoleEncryptionKey")?;
                        role_data.role_encryption_key = Some(KeyLocalRef::decode(d, ctx)?);
                    },
                    RoleDataInt::PaymentKey => {
                        role_data.payment_key =
                            Some(decode_helper(d, "PaymentKey in RoleData", ctx)?);
                    },
                }
            } else {
                if !(FIRST_ROLE_EXT_KEY..=LAST_ROLE_EXT_KEY).contains(&key) {
                    return Err(decode::Error::message(format!("Invalid role extended data key, should be with the range {FIRST_ROLE_EXT_KEY} - {LAST_ROLE_EXT_KEY}")));
                }
                role_data
                    .role_extended_data_keys
                    .insert(key, decode_any(d, "Role extended data keys")?);
            }
        }
        Ok(role_data)
    }
}
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
