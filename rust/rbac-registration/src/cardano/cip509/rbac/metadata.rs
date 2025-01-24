//! Cip509 RBAC metadata.

use std::collections::{HashMap, HashSet};

use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::{
    decode_any, decode_array_len, decode_bytes, decode_helper, decode_map_len,
};
use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use crate::{
    cardano::cip509::{
        decode_context::DecodeContext,
        rbac::{role_data::CborRoleData, C509Cert, SimplePublicKeyType, X509DerCert},
        utils::Cip0134UriSet,
        CertKeyHash, RoleData, RoleNumber,
    },
    utils::decode_helper::report_duplicated_key,
};

/// Cip509 RBAC metadata.
///
/// See [this document] for more details.
///
/// [this document]: https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX
#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct Cip509RbacMetadata {
    /// A potentially empty list of x509 certificates.
    pub x509_certs: Vec<X509DerCert>,
    /// A potentially empty list of c509 certificates.
    pub c509_certs: Vec<C509Cert>,
    /// A set of URIs contained in both x509 and c509 certificates.
    ///
    /// URIs from different certificate types are stored separately and certificate
    /// indexes are preserved too.
    ///
    /// This field isn't present in the encoded format and is populated by processing both
    /// `x509_certs` and `c509_certs` fields.
    pub certificate_uris: Cip0134UriSet,
    /// A list of public keys that can be used instead of storing full certificates.
    ///
    /// Check [this section] to understand how certificates and the public keys list are
    /// related.
    ///
    /// [this section]: https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX#storing-certificates-and-public-key
    pub pub_keys: Vec<SimplePublicKeyType>,
    /// A potentially empty list of revoked certificates.
    pub revocation_list: Vec<CertKeyHash>,
    /// A potentially empty role data.
    pub role_data: HashMap<RoleNumber, RoleData>,
    /// Optional map of purpose key data.
    /// Empty map if no purpose key data is present.
    pub purpose_key_data: HashMap<u16, Vec<u8>>,
}

/// The first valid purpose key.
const FIRST_PURPOSE_KEY: u16 = 200;
/// The last valid purpose key.
const LAST_PURPOSE_KEY: u16 = 299;

/// Enum of CIP509 RBAC metadata with its associated unsigned integer value.
#[derive(FromRepr, Debug, PartialEq, Copy, Clone)]
#[repr(u16)]
pub enum Cip509RbacMetadataInt {
    /// x509 certificates.
    X509Certs = 10,
    /// c509 certificates.
    C509Certs = 20,
    /// Public keys.
    PubKeys = 30,
    /// Revocation list.
    RevocationList = 40,
    /// Role data set.
    RoleSet = 100,
}

impl Decode<'_, DecodeContext<'_, '_>> for Cip509RbacMetadata {
    fn decode(d: &mut Decoder, decode_context: &mut DecodeContext) -> Result<Self, decode::Error> {
        let context = "Decoding Cip509RbacMetadata";

        let map_len = decode_map_len(d, context)?;

        let mut found_keys = Vec::new();

        let mut x509_certs = Vec::new();
        let mut c509_certs = Vec::new();
        let mut pub_keys = Vec::new();
        let mut revocation_list = Vec::new();
        let mut role_data = HashMap::new();
        let mut purpose_key_data = HashMap::new();

        for index in 0..map_len {
            let key: u16 = decode_helper(d, "key in Cip509RbacMetadata", &mut ())?;
            if let Some(key) = Cip509RbacMetadataInt::from_repr(key) {
                if report_duplicated_key(&found_keys, &key, index, context, decode_context.report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    Cip509RbacMetadataInt::X509Certs => {
                        match decode_array(
                            d,
                            "Cip509RbacMetadata x509 certificates",
                            decode_context.report,
                        ) {
                            Some(v) => x509_certs = v,
                            None => break,
                        }
                    },
                    Cip509RbacMetadataInt::C509Certs => {
                        match decode_array(
                            d,
                            "Cip509RbacMetadata c509 certificates",
                            decode_context.report,
                        ) {
                            Some(v) => c509_certs = v,
                            None => break,
                        }
                    },
                    Cip509RbacMetadataInt::PubKeys => {
                        match decode_array(
                            d,
                            "Cip509RbacMetadata public keys",
                            decode_context.report,
                        ) {
                            Some(v) => pub_keys = v,
                            None => break,
                        }
                    },
                    Cip509RbacMetadataInt::RevocationList => {
                        match decode_revocation_list(d, decode_context.report) {
                            Some(v) => revocation_list = v,
                            None => break,
                        }
                    },
                    Cip509RbacMetadataInt::RoleSet => {
                        if let Some(data) = decode_role_data(d, context, decode_context) {
                            role_data = data;
                        } else {
                            break;
                        }
                    },
                }
            } else {
                if !(FIRST_PURPOSE_KEY..=LAST_PURPOSE_KEY).contains(&key) {
                    decode_context.report.other(&format!("Invalid purpose key set ({key}), should be with the range {FIRST_PURPOSE_KEY} - {LAST_PURPOSE_KEY}"), context);
                    continue;
                }

                match decode_any(d, "purpose key") {
                    Ok(v) => {
                        purpose_key_data.insert(key, v.to_vec());
                    },
                    Err(e) => {
                        decode_context
                            .report
                            .other(&format!("Unable to decode purpose value: {e:?}"), context);
                        break;
                    },
                }
            }
        }

        let certificate_uris = Cip0134UriSet::new(&x509_certs, &c509_certs, decode_context.report);

        Ok(Self {
            x509_certs,
            c509_certs,
            certificate_uris,
            pub_keys,
            revocation_list,
            role_data,
            purpose_key_data,
        })
    }
}

/// Decodes an array of type T.
fn decode_array<'b, T>(
    d: &mut Decoder<'b>, context: &str, report: &mut ProblemReport,
) -> Option<Vec<T>>
where T: Decode<'b, ProblemReport> {
    let len = match decode_array_len(d, context) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Unable to decode array length: {e:?}"), context);
            return None;
        },
    };
    let len = match usize::try_from(len) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Invalid array length: {e:?}"), context);
            return Some(Vec::new());
        },
    };

    let mut result = Vec::with_capacity(len);
    for _ in 0..len {
        match T::decode(d, report) {
            Ok(v) => result.push(v),
            Err(e) => {
                report.other(&format!("Unable to decode array value: {e:?}"), context);
                return None;
            },
        }
    }
    Some(result)
}

/// Decode an array of revocation list.
fn decode_revocation_list(d: &mut Decoder, report: &ProblemReport) -> Option<Vec<CertKeyHash>> {
    let context = "Cip509RbacMetadata revocation list";
    let len = match decode_array_len(d, context) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Unable to decode array length: {e:?}"), context);
            return None;
        },
    };
    let len = match usize::try_from(len) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Invalid array length: {e:?}"), context);
            return Some(Vec::new());
        },
    };

    let mut result = Vec::with_capacity(len);
    for _ in 0..len {
        let bytes = match decode_bytes(d, context) {
            Ok(v) => v,
            Err(e) => {
                report.other(
                    &format!("Unable to decode certificate hash bytes: {e:?}"),
                    context,
                );
                return None;
            },
        };
        match CertKeyHash::try_from(bytes) {
            Ok(v) => result.push(v),
            Err(e) => {
                report.other(
                    &format!("Invalid revocation list certificate hash: {e:?}"),
                    context,
                );
            },
        }
    }
    Some(result)
}

/// Adds report entries if duplicated roles are found.
fn report_duplicated_roles(data: &[CborRoleData], context: &str, report: &ProblemReport) {
    let mut roles = HashSet::new();
    for role in data {
        let Some(number) = role.number else {
            continue;
        };
        if !roles.insert(number) {
            report.other(&format!("Duplicated role number {number:?} found"), context);
        }
    }
}

/// Decodes and converts a role data.
fn decode_role_data(
    d: &mut Decoder, context: &str, decode_context: &mut DecodeContext,
) -> Option<HashMap<RoleNumber, RoleData>> {
    let roles = decode_array(d, "Cip509RbacMetadata role set", decode_context.report)?;
    report_duplicated_roles(&roles, context, decode_context.report);
    let roles = roles
        .into_iter()
        .filter_map(|data| {
            if let Some(number) = data.number {
                Some((
                    number,
                    RoleData::new(data, decode_context.txn, decode_context.report),
                ))
            } else {
                None
            }
        })
        .collect();
    Some(roles)
}
