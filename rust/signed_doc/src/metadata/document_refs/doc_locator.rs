//! Document Locator, where a document can be located.
//! A [CBOR Encoded IPLD Content Identifier](https://github.com/ipld/cid-cbor/)
//! or also known as [IPFS CID](https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid).

use std::fmt::Display;

use minicbor::{Decode, Decoder, Encode};

/// CBOR tag of IPLD content identifiers (CIDs).
const CID_TAG: u64 = 42;

/// CID map key.
const CID_MAP_KEY: &str = "cid";

/// Document locator number of map item.
const DOC_LOC_MAP_ITEM: u64 = 1;

/// Document locator, no size limit.
#[derive(Clone, Debug, Default, PartialEq, Hash, Eq, serde::Serialize)]
pub struct DocLocator(Vec<u8>);

impl DocLocator {
    #[must_use]
    /// Length of the document locator.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    /// Is the document locator empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for DocLocator {
    fn from(value: Vec<u8>) -> Self {
        DocLocator(value)
    }
}

impl Display for DocLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cid: 0x{}", hex::encode(self.0.as_slice()))
    }
}

// document_locator = { "cid" => cid }
impl Decode<'_, ()> for DocLocator {
    fn decode(d: &mut Decoder, _ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocLocator decoding";

        let len = d.map()?.ok_or_else(|| {
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid map length"))
        })?;

        if len != DOC_LOC_MAP_ITEM {
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected map length {DOC_LOC_MAP_ITEM}, found {len}"
            )));
        }

        let key = d
            .str()
            .map_err(|e| e.with_message(format!("{CONTEXT}: expected string")))?;

        if key != "cid" {
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected key 'cid', found '{key}'"
            )));
        }

        let tag = d
            .tag()
            .map_err(|e| e.with_message(format!("{CONTEXT}: expected tag")))?;

        if tag.as_u64() != CID_TAG {
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected tag {CID_TAG}, found {tag}",
            )));
        }

        // No length limit
        let cid_bytes = d
            .bytes()
            .map_err(|e| e.with_message(format!("{CONTEXT}: expected bytes")))?;

        Ok(DocLocator(cid_bytes.to_vec()))
    }
}

impl Encode<()> for DocLocator {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(DOC_LOC_MAP_ITEM)?;
        e.str(CID_MAP_KEY)?;
        e.tag(minicbor::data::Tag::new(CID_TAG))?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use minicbor::{Decoder, Encoder};

    use super::*;

    #[test]
    fn test_doc_locator_encode_decode() {
        let locator = DocLocator(vec![1, 2, 3, 4]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut ()).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_loc = DocLocator::decode(&mut decoder, &mut ()).unwrap();
        assert_eq!(locator, decoded_doc_loc);
    }

    // Empty doc locator should not fail
    #[test]
    fn test_doc_locator_encode_decode_empty() {
        let locator = DocLocator(vec![]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut ()).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_loc = DocLocator::decode(&mut decoder, &mut ()).unwrap();
        assert_eq!(locator, decoded_doc_loc);
    }
}
