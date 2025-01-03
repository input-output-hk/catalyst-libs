//! Role Based Access Control (RBAC) metadata for CIP509.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-role-registration-metadata/CIP-XXXX/x509-roles.cddl>

pub mod certs;
pub mod pub_key;
pub mod role_data;
pub(crate) mod tag;

use std::collections::HashMap;

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
#[derive(FromRepr, Debug, PartialEq)]
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

impl Decode<'_, ()> for Cip509RbacMetadata {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "Cip509RbacMetadata")?;

        let mut x509_certs = Vec::new();
        let mut c509_certs = Vec::new();
        let mut pub_keys = Vec::new();
        let mut revocation_list = Vec::new();
        let mut role_set = Vec::new();
        let mut purpose_key_data = HashMap::new();

        for _ in 0..map_len {
            let key: u16 = decode_helper(d, "key in Cip509RbacMetadata", ctx)?;
            if let Some(key) = Cip509RbacMetadataInt::from_repr(key) {
                match key {
                    Cip509RbacMetadataInt::X509Certs => {
                        x509_certs = decode_array_rbac(d, "x509 certificate")?;
                    },
                    Cip509RbacMetadataInt::C509Certs => {
                        c509_certs = decode_array_rbac(d, "c509 certificate")?;
                    },
                    Cip509RbacMetadataInt::PubKeys => {
                        pub_keys = decode_array_rbac(d, "public keys")?;
                    },
                    Cip509RbacMetadataInt::RevocationList => {
                        revocation_list = decode_revocation_list(d)?;
                    },
                    Cip509RbacMetadataInt::RoleSet => {
                        role_set = decode_array_rbac(d, "role set")?;
                    },
                }
            } else {
                if !(FIRST_PURPOSE_KEY..=LAST_PURPOSE_KEY).contains(&key) {
                    return Err(decode::Error::message(format!("Invalid purpose key set, should be with the range {FIRST_PURPOSE_KEY} - {LAST_PURPOSE_KEY}")));
                }

                purpose_key_data.insert(key, decode_any(d, "purpose key")?);
            }
        }

        let certificate_uris = Cip0134UriSet::new(&x509_certs, &c509_certs).map_err(|e| {
            decode::Error::message(format!("Unable to parse URIs from certificates: {e:?}"))
        })?;

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

/// Decode an array of type T.
fn decode_array_rbac<'b, T>(d: &mut Decoder<'b>, from: &str) -> Result<Vec<T>, decode::Error>
where T: Decode<'b, ()> {
    let len = decode_array_len(d, &format!("{from} Cip509RbacMetadata"))?;
    let mut vec = Vec::with_capacity(usize::try_from(len).map_err(decode::Error::message)?);
    for _ in 0..len {
        vec.push(T::decode(d, &mut ())?);
    }
    Ok(vec)
}

/// Decode an array of revocation list.
fn decode_revocation_list(d: &mut Decoder) -> Result<Vec<CertKeyHash>, decode::Error> {
    let len = decode_array_len(d, "revocation list Cip509RbacMetadata")?;
    let mut revocation_list =
        Vec::with_capacity(usize::try_from(len).map_err(decode::Error::message)?);
    for _ in 0..len {
        let arr: [u8; 16] = decode_bytes(d, "revocation list Cip509RbacMetadata")?
            .try_into()
            .map_err(|_| decode::Error::message("Invalid revocation list size"))?;
        revocation_list.push(CertKeyHash::from(arr));
    }
    Ok(revocation_list)
}
