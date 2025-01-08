//! Role Based Access Control (RBAC) metadata for CIP509.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-role-registration-metadata/CIP-XXXX/x509-roles.cddl>

pub mod certs;
pub mod pub_key;
pub mod role_data;
pub(crate) mod tag;

use std::collections::HashMap;

use catalyst_types::problem_report::ProblemReport;
use certs::{C509Cert, X509DerCert};
use minicbor::{decode, Decode, Decoder};
use pub_key::SimplePublicKeyType;
use role_data::RoleData;
use strum_macros::FromRepr;

use super::types::cert_key_hash::CertKeyHash;
use crate::{
    cardano::cip509::utils::Cip0134UriSet,
    utils::decode_helper::{
        decode_any, decode_array_len, decode_bytes, decode_helper, decode_map_len,
        report_duplicated_key,
    },
};

/// Cip509 RBAC metadata.
///
/// See [this document] for more details.
///
/// [this document]: https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX
#[derive(Debug, PartialEq, Clone)]
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
    /// Check [this section] to understand the how certificates and the public keys list
    /// are related.
    ///
    /// [this section]: https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX#storing-certificates-and-public-key
    pub pub_keys: Vec<SimplePublicKeyType>,
    /// A potentially empty list of revoked certificates.
    pub revocation_list: Vec<CertKeyHash>,
    /// A potentially empty list of role data.
    // TODO: FIXME: decode into map.
    // (https://github.com/input-output-hk/catalyst-libs/pull/127#discussion_r1901557870)
    pub role_set: Vec<RoleData>,
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
enum Cip509RbacMetadataInt {
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

impl Decode<'_, ProblemReport> for Cip509RbacMetadata {
    fn decode(d: &mut Decoder, report: &mut ProblemReport) -> Result<Self, decode::Error> {
        let context = "Cip509RbacMetadata";
        let map_len = decode_map_len(d, context)?;

        let mut found_keys = Vec::new();

        let mut x509_certs = Vec::new();
        let mut c509_certs = Vec::new();
        let mut pub_keys = Vec::new();
        let mut revocation_list = Vec::new();
        let mut role_set = Vec::new();
        let mut purpose_key_data = HashMap::new();

        for index in 0..map_len {
            let key: u16 = decode_helper(d, "key in Cip509RbacMetadata", &mut ())?;
            if let Some(key) = Cip509RbacMetadataInt::from_repr(key) {
                if report_duplicated_key(&found_keys, &key, index, context, report) {
                    continue;
                }
                found_keys.push(key);

                match key {
                    Cip509RbacMetadataInt::X509Certs => {
                        x509_certs =
                            decode_array(d, "Cip509RbacMetadata x509 certificates", report);
                    },
                    Cip509RbacMetadataInt::C509Certs => {
                        c509_certs = decode_array(d, "Cip509RbacMetadata c509 certificate", report);
                    },
                    Cip509RbacMetadataInt::PubKeys => {
                        pub_keys = decode_array(d, "Cip509RbacMetadata public keys", report);
                    },
                    Cip509RbacMetadataInt::RevocationList => {
                        revocation_list = decode_revocation_list(d, report);
                    },
                    Cip509RbacMetadataInt::RoleSet => {
                        role_set = decode_array(d, "Cip509RbacMetadata role set", report);
                    },
                }
            } else {
                if !(FIRST_PURPOSE_KEY..=LAST_PURPOSE_KEY).contains(&key) {
                    report.other(&format!("Invalid purpose key set, should be with the range {FIRST_PURPOSE_KEY} - {LAST_PURPOSE_KEY}"), context);
                    continue;
                }

                match decode_any(d, "purpose key") {
                    Ok(v) => {
                        purpose_key_data.insert(key, v);
                    },
                    Err(e) => {
                        report.other(&format!("Unable to decode purpose value: {e:?}"), context);
                    },
                }
            }
        }

        let certificate_uris = Cip0134UriSet::new(&x509_certs, &c509_certs, report);

        Ok(Self {
            x509_certs,
            c509_certs,
            certificate_uris,
            pub_keys,
            revocation_list,
            role_set,
            purpose_key_data,
        })
    }
}

/// Decodes an array of type T.
fn decode_array<'b, T>(d: &mut Decoder<'b>, context: &str, report: &ProblemReport) -> Vec<T>
where T: Decode<'b, ()> {
    let len = match decode_array_len(d, context) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Unable to decode array length: {e:?}"), context);
            return Vec::new();
        },
    };
    let len = match usize::try_from(len) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Invalid array length: {e:?}"), context);
            return Vec::new();
        },
    };

    let mut result = Vec::with_capacity(len);
    for _ in 0..len {
        match T::decode(d, &mut ()) {
            Ok(v) => result.push(v),
            Err(e) => {
                report.other(&format!("Unable to decode array value: {e:?}"), context);
            },
        }
    }
    result
}

/// Decode an array of revocation list.
fn decode_revocation_list(d: &mut Decoder, report: &ProblemReport) -> Vec<CertKeyHash> {
    let context = "Cip509RbacMetadata revocation list";
    let len = match decode_array_len(d, context) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Unable to decode array length: {e:?}"), context);
            return Vec::new();
        },
    };
    let len = match usize::try_from(len) {
        Ok(v) => v,
        Err(e) => {
            report.other(&format!("Invalid array length: {e:?}"), context);
            return Vec::new();
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
                continue;
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
    result
}
