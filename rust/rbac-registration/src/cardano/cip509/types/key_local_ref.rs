//! A local key reference.

use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::decode_helper;
use minicbor::{Decode, Decoder, decode};
use strum_macros::FromRepr;

use crate::cardano::cip509::rbac::Cip509RbacMetadataInt;

/// Local key reference.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct KeyLocalRef {
    /// Local reference.
    pub local_ref: LocalRefInt,
    /// Key offset.
    pub key_offset: usize,
}

/// Enum of local reference with its associated unsigned integer value.
#[derive(FromRepr, Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[repr(u8)]
pub enum LocalRefInt {
    /// x509 certificates.
    X509Certs = Cip509RbacMetadataInt::X509Certs as u8, // 10
    /// c509 certificates.
    C509Certs = Cip509RbacMetadataInt::C509Certs as u8, // 20
    /// Public keys.
    PubKeys = Cip509RbacMetadataInt::PubKeys as u8, // 30
}

impl Decode<'_, ProblemReport> for KeyLocalRef {
    fn decode(
        d: &mut Decoder,
        report: &mut ProblemReport,
    ) -> Result<Self, decode::Error> {
        let local_ref =
            LocalRefInt::from_repr(decode_helper(d, "LocalRef in KeyLocalRef", &mut ())?)
                .ok_or(decode::Error::message("Invalid local reference"))?;
        let key_offset: u64 = decode_helper(d, "KeyOffset in KeyLocalRef", &mut ())?;
        let key_offset = if let Ok(v) = usize::try_from(key_offset) {
            v
        } else {
            report.invalid_value(
                "key_offset",
                &format!("{key_offset}"),
                &format!("Value must be less than {}", usize::MAX),
                "KeyLocalRef decoding",
            );
            0
        };
        Ok(Self {
            local_ref,
            key_offset,
        })
    }
}
