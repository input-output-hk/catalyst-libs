//! Certificates for the RBAC metadata.

use c509_certificate::c509::C509;
use minicbor::{decode, Decode, Decoder};
use x509_cert::{der::Decode as x509Decode, Certificate};

use crate::utils::decode_helper::{decode_array_len, decode_bytes, decode_helper, decode_tag};

use super::tag::KeyTag;

// ------------------x509------------------------

/// Enum of possible X.509 DER certificate.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum X509DerCert {
    /// Undefined indicates skipped element.
    #[default]
    Undefined,
    /// Deleted indicates the key is deleted.
    Deleted,
    /// X.509 certificate.
    X509Cert(Vec<u8>),
}

impl Decode<'_, ()> for X509DerCert {
    fn decode(d: &mut Decoder, _ctx: &mut ()) -> Result<Self, decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Tag => {
                let tag = decode_tag(d, "X509DerCert")?;
                match tag {
                    t if t == KeyTag::Deleted.tag() => Ok(Self::Deleted),
                    _ => Err(decode::Error::message("Unknown tag for X509DerCert")),
                }
            },
            minicbor::data::Type::Undefined => Ok(Self::Undefined),
            minicbor::data::Type::Bytes => {
                let data = decode_bytes(d, "X509DerCert")?;
                Certificate::from_der(&data)
                    .map_err(|_| decode::Error::message("Invalid x509 certificate"))?;
                Ok(Self::X509Cert(data.clone()))
            },
            _ => Err(decode::Error::message("Invalid datatype for X509DerCert")),
        }
    }
}

// ------------------c509-----------------------

/// Enum of possible X.509 DER certificate.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum C509Cert {
    /// Undefined indicates skipped element.
    #[default]
    Undefined,
    /// Deleted indicates the key is deleted.
    Deleted,
    /// A c509 certificate in metadatum reference.
    C509CertInMetadatumReference(C509CertInMetadatumReference),
    /// A c509 certificate.
    C509Certificate(Box<C509>),
}

impl Decode<'_, ()> for C509Cert {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Tag => {
                let tag = decode_tag(d, "C509Cert")?;
                match tag {
                    t if t == KeyTag::Deleted.tag() => Ok(Self::Deleted),
                    _ => Err(decode::Error::message("Unknown tag for C509Cert")),
                }
            },
            minicbor::data::Type::Array => {
                let arr_len = decode_array_len(d, "C509Cert")?;
                // C509CertInMetadatumReference must have 3 items
                if arr_len == 3 {
                    Ok(Self::C509CertInMetadatumReference(
                        C509CertInMetadatumReference::decode(d, ctx)?,
                    ))
                } else {
                    Err(decode::Error::message(
                        "Invalid length C509CertInMetadatumReference, expected 3",
                    ))
                }
            },
            minicbor::data::Type::Bytes => {
                let c509 = decode_bytes(d, "C509Cert")?;
                let mut c509_d = Decoder::new(&c509);
                Ok(Self::C509Certificate(Box::new(C509::decode(
                    &mut c509_d,
                    ctx,
                )?)))
            },
            minicbor::data::Type::Undefined => Ok(Self::Undefined),
            _ => Err(decode::Error::message("Invalid datatype for C509Cert")),
        }
    }
}

/// A struct of c509 certificate in metadatum reference.
#[derive(Debug, PartialEq, Clone)]
pub struct C509CertInMetadatumReference {
    /// Transaction output field.
    txn_output_field: u8,
    /// Transaction output index.
    txn_output_index: u64,
    /// Optional certificate reference.
    cert_ref: Option<Vec<u64>>,
}

impl Decode<'_, ()> for C509CertInMetadatumReference {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let txn_output_field: u8 =
            decode_helper(d, "txn output field in C509CertInMetadatumReference", ctx)?;
        let txn_output_index: u64 =
            decode_helper(d, "txn output index in C509CertInMetadatumReference", ctx)?;
        let cert_ref = match d.datatype()? {
            minicbor::data::Type::Array => {
                let len = decode_array_len(d, "cert ref in C509CertInMetadatumReference")?;
                let arr: Result<Vec<u64>, _> = (0..len).map(|_| d.u64()).collect();
                arr.map(Some)
            },
            minicbor::data::Type::Null => Ok(None),
            _ => Ok(Some(vec![decode_helper(
                d,
                "C509CertInMetadatumReference",
                ctx,
            )?])),
        }?;
        Ok(Self {
            txn_output_field,
            txn_output_index,
            cert_ref,
        })
    }
}
