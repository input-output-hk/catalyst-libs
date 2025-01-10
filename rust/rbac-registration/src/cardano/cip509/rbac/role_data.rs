//! Role data for RBAC metadata.

use std::collections::HashMap;

use catalyst_types::problem_report::ProblemReport;
use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use crate::{
    cardano::cip509::rbac::{Cip509RbacMetadataInt, RoleNumber},
    utils::decode_helper::{
        decode_any, decode_array_len, decode_helper, decode_map_len, report_duplicated_key,
    },
};

/// Role data as it encoded in CBOR.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct RoleData {
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
#[derive(FromRepr, Debug, PartialEq, Copy, Clone)]
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

/// A wrapper over role number and data. Only needed to decode from CBOR.
pub struct RoleNumberAndData {
    /// A role number.
    pub number: RoleNumber,
    /// A role data.
    pub data: RoleData,
}

impl Decode<'_, ProblemReport> for RoleNumberAndData {
    fn decode(d: &mut Decoder, report: &mut ProblemReport) -> Result<Self, decode::Error> {
        let context = "Decoding role data";
        let map_len = decode_map_len(d, "RoleData")?;

        let mut found_keys = Vec::new();

        let mut data = RoleData::default();
        let mut number: u8 = 0;

        for index in 0..map_len {
            let key: u8 = decode_helper(d, "key in RoleData", &mut ())?;
            if let Some(key) = RoleDataInt::from_repr(key) {
                if report_duplicated_key(&found_keys, &key, index, context, report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    RoleDataInt::RoleNumber => {
                        match decode_helper(d, "RoleNumber in RoleData", &mut ()) {
                            Ok(v) => number = v,
                            Err(e) => {
                                report.other(
                                    &format!("Unable to decode role number: {e:?}"),
                                    context,
                                );
                            },
                        }
                    },
                    RoleDataInt::RoleSigningKey => {
                        if let Err(e) = decode_array_len(d, "RoleSigningKey") {
                            report.other(&format!("{e:?}"), context);
                            continue;
                        }

                        match KeyLocalRef::decode(d, &mut ()) {
                            Ok(v) => data.role_signing_key = Some(v),
                            Err(e) => {
                                report.other(
                                    &format!("Unable to decode role signing key: {e:?}"),
                                    context,
                                );
                            },
                        }
                    },
                    RoleDataInt::RoleEncryptionKey => {
                        if let Err(e) = decode_array_len(d, "RoleEncryptionKey") {
                            report.other(&format!("{e:?}"), context);
                            continue;
                        }

                        match KeyLocalRef::decode(d, &mut ()) {
                            Ok(v) => data.role_encryption_key = Some(v),
                            Err(e) => {
                                report.other(
                                    &format!("Unable to decode role encryption key: {e:?}"),
                                    context,
                                );
                            },
                        }
                    },
                    RoleDataInt::PaymentKey => {
                        data.payment_key =
                            Some(decode_helper(d, "PaymentKey in RoleData", &mut ())?);
                    },
                }
            } else {
                if !(FIRST_ROLE_EXT_KEY..=LAST_ROLE_EXT_KEY).contains(&key) {
                    report.other(&format!("Invalid role extended data key ({key}), should be with the range {FIRST_ROLE_EXT_KEY} - {LAST_ROLE_EXT_KEY}"), context);
                    continue;
                }
                data.role_extended_data_keys
                    .insert(key, decode_any(d, "Role extended data keys")?);
            }
        }
        Ok(RoleNumberAndData {
            number: number.into(),
            data,
        })
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
