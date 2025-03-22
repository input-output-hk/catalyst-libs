//! X509 certificate.

use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::{decode_bytes, decode_tag};
use minicbor::{decode, Decode, Decoder};
use x509_cert::{der::Decode as x509Decode, Certificate};

use crate::cardano::cip509::rbac::tag::KeyTag;

/// Enum of possible X.509 DER certificate.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Default)]
pub enum X509DerCert {
    /// Undefined indicates skipped element.
    #[default]
    Undefined,
    /// Deleted indicates the key is deleted.
    Deleted,
    /// X.509 certificate.
    X509Cert(Box<Certificate>),
}

impl Decode<'_, ProblemReport> for X509DerCert {
    fn decode(d: &mut Decoder, _report: &mut ProblemReport) -> Result<Self, decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Tag => {
                let tag = decode_tag(d, "X509DerCert")?;
                match tag {
                    t if t == KeyTag::Deleted.tag() => {
                        d.undefined()?;
                        Ok(Self::Deleted)
                    },
                    _ => Err(decode::Error::message("Unknown tag for X509DerCert")),
                }
            },
            minicbor::data::Type::Undefined => {
                d.undefined()?;
                Ok(Self::Undefined)
            },
            minicbor::data::Type::Bytes => {
                let data = decode_bytes(d, "X509DerCert")?;
                let certificate = Certificate::from_der(&data).map_err(|e| {
                    decode::Error::message(format!("Invalid x509 certificate: {e:?}"))
                })?;
                Ok(Self::X509Cert(Box::new(certificate)))
            },
            _ => Err(decode::Error::message("Invalid datatype for X509DerCert")),
        }
    }
}
