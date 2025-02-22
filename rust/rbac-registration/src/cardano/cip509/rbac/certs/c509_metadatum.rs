//! C509 certificate in metadatum reference.

use cbork_utils::decode_helper::{decode_array_len, decode_helper};
use minicbor::{Decode, Decoder, decode};

/// C509 certificate in metadatum reference.
#[derive(Debug, PartialEq, Clone)]
pub struct C509CertInMetadatumReference {
    /// Transaction output field.
    pub txn_output_field: u8,
    /// Transaction output index.
    pub txn_output_index: u64,
    /// Optional certificate reference.
    pub cert_ref: Option<Vec<u64>>,
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
            _ => {
                Ok(Some(vec![decode_helper(
                    d,
                    "C509CertInMetadatumReference",
                    ctx,
                )?]))
            },
        }?;
        Ok(Self {
            txn_output_field,
            txn_output_index,
            cert_ref,
        })
    }
}
