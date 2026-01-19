//! Document Locator, where a document can be located.
//! A [CBOR Encoded IPLD Content Identifier](https://github.com/ipld/cid-cbor/)
//! or also known as [IPFS CID](https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid).

use std::{fmt::Display, ops::Deref, str::FromStr};

use cbork_utils::{decode_context::DecodeCtx, map::Map};
use minicbor::{Decode, Decoder, Encode};

use crate::{
    cid_v1::{Cid, CidError},
    metadata::document_refs::DocRefError,
};

/// CID map key.
const CID_MAP_KEY: &str = "cid";

/// Document locator number of map item.
const DOC_LOC_MAP_ITEM: u64 = 1;

/// Document locator wrapping a CID (Content Identifier).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocLocator(Cid);

impl Deref for DocLocator {
    type Target = Cid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Cid> for DocLocator {
    fn from(cid: Cid) -> Self {
        Self(cid)
    }
}

impl TryFrom<&[u8]> for DocLocator {
    type Error = CidError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let cid = Cid::try_from(bytes)?;
        Ok(Self(cid))
    }
}

impl Display for DocLocator {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DocLocator {
    type Err = DocRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cid = Cid::from_str(s).map_err(|e| DocRefError::StringConversion(e.to_string()))?;
        Ok(Self(cid))
    }
}

impl<'de> serde::Deserialize<'de> for DocLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<DocLocator>().map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for DocLocator {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// document_locator = { "cid" => tag(42)(cid_bytes) }
impl Decode<'_, ()> for DocLocator {
    fn decode(
        d: &mut Decoder,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocLocator decoding";

        let entries = Map::decode(d, &mut DecodeCtx::Deterministic)?;

        match entries.as_slice() {
            [entry] => {
                let key = minicbor::Decoder::new(&entry.key_bytes)
                    .str()
                    .map_err(|e| e.with_message(format!("{CONTEXT}: expected string")))?;

                if key != "cid" {
                    return Err(minicbor::decode::Error::message(format!(
                        "{CONTEXT}: expected key 'cid', found '{key}'"
                    )));
                }

                let mut value_decoder = minicbor::Decoder::new(&entry.value);

                // Decode the Cid, which validates tag(42) and CID format
                let cid = Cid::decode(&mut value_decoder, &mut ()).map_err(|e| {
                    let msg = format!("{CONTEXT}: {e}");
                    e.with_message(msg)
                })?;

                Ok(DocLocator(cid))
            },
            _ => {
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: expected map length {DOC_LOC_MAP_ITEM}, found {}",
                    entries.len()
                )))
            },
        }
    }
}

impl Encode<()> for DocLocator {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(DOC_LOC_MAP_ITEM)?;
        e.str(CID_MAP_KEY)?;
        // Delegate Cid encoding which handles tag(42) and CID bytes
        self.0.encode(e, ctx)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use catalyst_types::uuid::{UuidV4, UuidV7};
    use minicbor::{Decoder, Encoder};

    use super::*;
    use crate::{ContentType, builder::Builder, tests_utils::create_dummy_doc_ref};

    #[test]
    fn test_doc_locator_encode_decode() {
        let locator = create_dummy_doc_ref().doc_locator().clone();
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut ()).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_loc = DocLocator::decode(&mut decoder, &mut ()).unwrap();
        assert_eq!(locator, decoded_doc_loc);
    }

    #[test]
    fn test_doc_locator_display() {
        let locator = create_dummy_doc_ref().doc_locator().clone();
        let display_str = locator.to_string();
        assert!(
            display_str.starts_with('b'),
            "Should use multibase format starting with 'b'"
        );
    }

    #[test]
    fn test_doc_locator_from_str() {
        let locator = create_dummy_doc_ref().doc_locator().clone();
        let display_str = locator.to_string();
        let parsed = display_str
            .parse::<DocLocator>()
            .expect("Should parse multibase string");
        assert_eq!(locator, parsed);
    }

    #[test]
    fn test_doc_locator_from_cid() {
        let id = UuidV7::new();
        let ver = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": id.to_string(),
                "ver": ver.to_string(),
                "type": UuidV4::new().to_string(),
                "content-type": ContentType::Json,
            }))
            .expect("Should create metadata")
            .with_json_content(&serde_json::json!({"test": "content"}))
            .expect("Should set content")
            .build()
            .expect("Should build document");

        let cid = doc.to_cid_v1().expect("Should generate CID");
        let locator = DocLocator::from(cid);

        assert_eq!(&*locator, &cid);
    }
}
