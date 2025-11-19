//! Role data for RBAC metadata.

use std::collections::HashMap;

use catalyst_types::{
    catalyst_id::role_index::RoleId,
    cbor_utils::{report_duplicated_key, report_missing_keys},
    problem_report::ProblemReport,
};
use cbork_utils::decode_helper::{decode_any, decode_array_len, decode_helper, decode_map_len};
use minicbor::{Decode, Decoder, decode};
use strum_macros::FromRepr;

use crate::cardano::cip509::KeyLocalRef;

/// Role data as encoded in CBOR.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CborRoleData {
    /// A role number.
    pub number: Option<RoleId>,
    /// Optional role signing key.
    pub signing_key: Option<KeyLocalRef>,
    /// Optional role encryption key.
    pub encryption_key: Option<KeyLocalRef>,
    /// Optional payment key.
    pub payment_key: Option<u16>,
    /// Optional role extended data keys.
    /// Empty map if no role extended data keys.
    pub extended_data: HashMap<u8, Vec<u8>>,
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
    RoleId = 0,
    /// Role signing key.
    RoleSigningKey = 1,
    /// Role encryption key.
    RoleEncryptionKey = 2,
    /// Payment key.
    PaymentKey = 3,
}

impl Decode<'_, ProblemReport> for CborRoleData {
    fn decode(
        d: &mut Decoder,
        report: &mut ProblemReport,
    ) -> Result<Self, decode::Error> {
        let context = "Decoding role data";
        let map_len = decode_map_len(d, "RoleData")?;

        let mut found_keys = Vec::new();

        let mut data = CborRoleData::default();

        for index in 0..map_len {
            let key: u8 = decode_helper(d, "key in RoleData", &mut ())?;
            if let Some(key) = RoleDataInt::from_repr(key) {
                if report_duplicated_key(&found_keys, &key, index, "RoleData", report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    RoleDataInt::RoleId => {
                        match decode_helper::<RoleId, _>(d, "RoleId in RoleData", &mut ()) {
                            Ok(v) => data.number = Some(v),
                            Err(e) => {
                                report.other(
                                    &format!("Unable to decode role number: {e:?}"),
                                    context,
                                );
                                break;
                            },
                        }
                    },
                    RoleDataInt::RoleSigningKey => {
                        match decode_signing_key(d, context, report) {
                            Ok(v) => data.signing_key = v,
                            Err(()) => break,
                        }
                    },
                    RoleDataInt::RoleEncryptionKey => {
                        match decode_encryption_key(d, context, report) {
                            Ok(v) => data.encryption_key = v,
                            Err(()) => break,
                        }
                    },
                    RoleDataInt::PaymentKey => {
                        match decode_helper(d, "PaymentKey in RoleData", &mut ()) {
                            Ok(v) => data.payment_key = Some(v),
                            Err(e) => {
                                report.other(
                                    &format!("Unable to decode role payment key: {e:?}"),
                                    context,
                                );
                                break;
                            },
                        }
                    },
                }
            } else {
                if !(FIRST_ROLE_EXT_KEY..=LAST_ROLE_EXT_KEY).contains(&key) {
                    report.other(&format!("Invalid role extended data key ({key}), should be with the range {FIRST_ROLE_EXT_KEY} - {LAST_ROLE_EXT_KEY}"), context);
                    continue;
                }
                let value = match decode_any(d, "Role extended data keys") {
                    Ok(v) => v,
                    Err(e) => {
                        report.other(
                            &format!("Unable to decode role extended data for {key} key: {e:?}"),
                            context,
                        );
                        continue;
                    },
                };
                if data.extended_data.insert(key, value.to_vec()).is_some() {
                    report.other(
                        &format!("Duplicated {key} key in the role extended data"),
                        context,
                    );
                }
            }
        }

        let required_keys = [RoleDataInt::RoleId];
        report_missing_keys(&found_keys, &required_keys, context, report);

        Ok(data)
    }
}

/// Decodes a signing key.
fn decode_signing_key(
    d: &mut Decoder,
    context: &str,
    report: &mut ProblemReport,
) -> Result<Option<KeyLocalRef>, ()> {
    if let Err(e) = decode_array_len(d, "RoleSigningKey") {
        report.other(&format!("{e:?}"), context);
        return Err(());
    }

    match KeyLocalRef::decode(d, report) {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            report.other(
                &format!("Unable to decode role signing key: {e:?}"),
                context,
            );
            Ok(None)
        },
    }
}

/// Decodes an encryption key.
fn decode_encryption_key(
    d: &mut Decoder,
    context: &str,
    report: &mut ProblemReport,
) -> Result<Option<KeyLocalRef>, ()> {
    if let Err(e) = decode_array_len(d, "RoleEncryptionKey") {
        report.other(&format!("{e:?}"), context);
        return Err(());
    }

    match KeyLocalRef::decode(d, report) {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            report.other(
                &format!("Unable to decode role encryption key: {e:?}"),
                context,
            );
            Ok(None)
        },
    }
}
