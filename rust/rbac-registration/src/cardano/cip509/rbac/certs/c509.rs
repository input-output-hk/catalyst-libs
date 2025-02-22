//! C509 certificate.

use c509_certificate::c509::C509;
use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::{decode_array_len, decode_bytes, decode_tag};
use minicbor::{Decode, Decoder, decode};

use crate::cardano::cip509::rbac::{certs::C509CertInMetadatumReference, tag::KeyTag};

/// An enum of possible C509 certificate values.
#[allow(clippy::module_name_repetitions)]
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

impl Decode<'_, ProblemReport> for C509Cert {
    fn decode(d: &mut Decoder, _report: &mut ProblemReport) -> Result<Self, decode::Error> {
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
                        C509CertInMetadatumReference::decode(d, &mut ())?,
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
                    &mut (),
                )?)))
            },
            minicbor::data::Type::Undefined => {
                d.undefined()?;
                Ok(Self::Undefined)
            },
            _ => Err(decode::Error::message("Invalid datatype for C509Cert")),
        }
    }
}
