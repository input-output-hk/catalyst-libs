//! Document Locator, where a document can be located.
//! A [CBOR Encoded IPLD Content Identifier](https://github.com/ipld/cid-cbor/)
//! or also known as [IPFS CID](https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid).

use std::fmt::Display;

use catalyst_types::problem_report::ProblemReport;
use coset::cbor::Value;
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

impl From<DocLocator> for Value {
    fn from(value: DocLocator) -> Self {
        Value::Map(vec![(
            Value::Text(CID_MAP_KEY.to_string()),
            Value::Tag(CID_TAG, Box::new(Value::Bytes(value.0.clone()))),
        )])
    }
}

// document_locator = { "cid" => cid }
impl Decode<'_, ProblemReport> for DocLocator {
    fn decode(
        d: &mut Decoder, report: &mut ProblemReport,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocLocator decoding";

        let len = d.map()?.ok_or_else(|| {
            report.invalid_value("Map", "Invalid length", "Valid length", CONTEXT);
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid map length"))
        })?;

        if len != DOC_LOC_MAP_ITEM {
            report.invalid_value(
                "Map length",
                &len.to_string(),
                &DOC_LOC_MAP_ITEM.to_string(),
                CONTEXT,
            );
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected map length {DOC_LOC_MAP_ITEM}, found {len}"
            )));
        }

        let key = d.str().map_err(|e| {
            report.invalid_value("Key", "Not a string", "String", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected string"))
        })?;

        if key != "cid" {
            report.invalid_value("Key", key, "'cid'", CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected key 'cid', found '{key}'"
            )));
        }

        let tag = d.tag().map_err(|e| {
            report.invalid_value("CBOR tag", "Invalid tag", "Valid tag", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected tag"))
        })?;

        if tag.as_u64() != CID_TAG {
            report.invalid_value("CBOR tag", &tag.to_string(), &CID_TAG.to_string(), CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected tag {CID_TAG}, found {tag}",
            )));
        }

        // No length limit
        let cid_bytes = d.bytes().map_err(|e| {
            report.invalid_value("CID bytes", "Invalid bytes", "Valid bytes", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected bytes"))
        })?;

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
        let mut report = ProblemReport::new("Test doc locator");
        let locator = DocLocator(vec![1, 2, 3, 4]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut ()).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_loc = DocLocator::decode(&mut decoder, &mut report).unwrap();
        assert_eq!(locator, decoded_doc_loc);
    }

    // Empty doc locator should not fail
    #[test]
    fn test_doc_locator_encode_decode_empty() {
        let mut report = ProblemReport::new("Test doc locator empty");
        let locator = DocLocator(vec![]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut ()).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_loc = DocLocator::decode(&mut decoder, &mut report).unwrap();
        assert_eq!(locator, decoded_doc_loc);
    }

    #[test]
    #[allow(clippy::indexing_slicing)]
    fn test_doc_locator_to_value() {
        let data = vec![1, 2, 3, 4];
        let locator = DocLocator(data.clone());
        let value: Value = locator.into();
        let map = value.into_map().unwrap();
        assert_eq!(map.len(), usize::try_from(DOC_LOC_MAP_ITEM).unwrap());
        let key = map[0].0.clone().into_text().unwrap();
        assert_eq!(key, CID_MAP_KEY);
        let (tag, value) = map[0].1.clone().into_tag().unwrap();
        assert_eq!(tag, CID_TAG);
        assert_eq!(value.into_bytes().unwrap(), data);
    }
}
