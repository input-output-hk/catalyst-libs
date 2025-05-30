use std::fmt::Display;

use minicbor::Decode;

use crate::DecodeContext;
const CID_TAG: u64 = 42;

#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocLocator(Vec<u8>);

impl From<Vec<u8>> for DocLocator {
    fn from(value: Vec<u8>) -> Self {
        DocLocator(value)
    }
}
impl Display for DocLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cid: {}", hex::encode(self.0.as_slice()))
    }
}

// document_locator = { "cid" => cid }
impl<'b> Decode<'b, DecodeContext<'_>> for DocLocator {
    fn decode(
        d: &mut minicbor::Decoder<'b>, ctx: &mut DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocLocator decoding";

        let len = d.map()?.ok_or_else(|| {
            ctx.report
                .invalid_value("map", "invalid map", "valid map", CONTEXT);
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid map"))
        })?;

        if len != 1 {
            ctx.report
                .invalid_value("map length", &len.to_string(), "1", CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected map length 1, found {len}"
            )));
        }

        let key = d.str().map_err(|e| {
            ctx.report
                .invalid_value("Key", "not a string", "String", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected string"))
        })?;

        if key != "cid" {
            ctx.report.invalid_value("Key", key, "'cid'", CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected key 'cid', found '{key}'"
            )));
        }

        let tag = d.tag().map_err(|e| {
            ctx.report
                .invalid_value("CBOR tag", "invalid tag", "valid tag", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected tag"))
        })?;

        if tag.as_u64() != CID_TAG {
            ctx.report
                .invalid_value("CBOR tag", &tag.to_string(), &CID_TAG.to_string(), CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected tag {}, found {}",
                CID_TAG, tag
            )));
        }

        // No length limit
        let cid_bytes = d.bytes().map_err(|e| {
            ctx.report
                .invalid_value("CID bytes", "invalid bytes", "valid bytes", CONTEXT);
            ("Unable to decode CID bytes", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected bytes"))
        })?;

        Ok(DocLocator(cid_bytes.to_vec()))
    }
}
