//! C509 Certificate

use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

use crate::{
    cert_tbs::TbsCert,
    helper::{
        decode::{decode_bytes, decode_datatype},
        encode::{encode_bytes, encode_null},
    },
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
/// A struct represents the `C509` Certificate.
pub struct C509 {
    /// A TBS Certificate.
    tbs_cert: TbsCert,
    /// An optional `IssuerSignatureValue` of the C509 Certificate.
    issuer_signature_value: Option<Vec<u8>>,
}

impl C509 {
    /// Create a new instance of C509 Certificate .
    #[must_use]
    pub fn new(tbs_cert: TbsCert, issuer_signature_value: Option<Vec<u8>>) -> Self {
        Self {
            tbs_cert,
            issuer_signature_value,
        }
    }

    /// Get the `TBSCertificate` of the C509 Certificate.
    #[must_use]
    pub fn tbs_cert(&self) -> &TbsCert {
        &self.tbs_cert
    }

    /// Get the `IssuerSignatureValue` of the C509 Certificate.
    #[must_use]
    pub fn issuer_signature_value(&self) -> &Option<Vec<u8>> {
        &self.issuer_signature_value
    }
}

impl Encode<()> for C509 {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.tbs_cert.encode(e, ctx)?;
        match self.issuer_signature_value {
            Some(ref value) => encode_bytes(e, "C509 Issuer Signature value", value)?,
            None => encode_null(e, "C509 Issuer Signature value")?,
        }
        Ok(())
    }
}

impl Decode<'_, ()> for C509 {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let tbs_cert = TbsCert::decode(d, ctx)?;
        let issuer_signature_value = match decode_datatype(d, "C509 Issuer Signature value")? {
            minicbor::data::Type::Bytes => Some(decode_bytes(d, "C509 Issuer Signature value")?),
            _ => None,
        };
        Ok(Self::new(tbs_cert, issuer_signature_value))
    }
}
