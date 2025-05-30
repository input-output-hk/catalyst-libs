/// Document Locator, where a document can be located.
/// A [CBOR Encoded IPLD Content Identifier](https://github.com/ipld/cid-cbor/)
/// or also known as [IPFS CID](https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid).
use std::fmt::Display;

use catalyst_types::problem_report::ProblemReport;
use minicbor::{Decode, Decoder, Encode};

// CBOR tag of IPLD content identifiers (CIDs)
const CID_TAG: u64 = 42;

// Document locator.
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
impl Decode<'_, ProblemReport> for DocLocator {
    fn decode(
        d: &mut Decoder, report: &mut ProblemReport,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocLocator decoding";

        let len = d.map()?.ok_or_else(|| {
            report
                .invalid_value("map", "invalid map length", "valid map length", CONTEXT);
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid map length"))
        })?;

        if len != 1 {
            report
                .invalid_value("map length", &len.to_string(), "1", CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected map length 1, found {len}"
            )));
        }

        let key = d.str().map_err(|e| {
            report
                .invalid_value("Key", "Not a string", "String", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected string"))
        })?;

        if key != "cid" {
            report.invalid_value("Key", key, "'cid'", CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected key 'cid', found '{key}'"
            )));
        }

        let tag = d.tag().map_err(|e| {
            report
                .invalid_value("CBOR tag", "invalid tag", "valid tag", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected tag"))
        })?;

        if tag.as_u64() != CID_TAG {
            report
                .invalid_value("CBOR tag", &tag.to_string(), &CID_TAG.to_string(), CONTEXT);
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected tag {}, found {}",
                CID_TAG, tag
            )));
        }

        // No length limit
        let cid_bytes = d.bytes().map_err(|e| {
            report
                .invalid_value("CID bytes", "invalid bytes", "valid bytes", CONTEXT);
            ("Unable to decode CID bytes", CONTEXT);
            e.with_message(format!("{CONTEXT}: expected bytes"))
        })?;

        Ok(DocLocator(cid_bytes.to_vec()))
    }
}

impl Encode<ProblemReport> for DocLocator {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, report: &mut ProblemReport,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        const CONTEXT: &str = "DocLocator encoding";
        e.tag(minicbor::data::Tag::new(CID_TAG));
        e.bytes(&self.0).map_err(|_| {
            report.invalid_encoding("CID", "invalid bytes", "valid CBOR byte", CONTEXT);
            minicbor::encode::Error::message(format!("{CONTEXT}: Unable to encode CID bytes"))
        })?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(warnings)]

mod tests {

    use minicbor::{Decoder, Encoder};

    use super::*;

    #[test]
    fn test_doc_locator() {
        let mut report = ProblemReport::new("Test doc locator");
        let locator = DocLocator(vec![1, 2, 3, 4]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut report).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded = DocLocator::decode(&mut decoder, &mut report).unwrap();
        assert_eq!(locator, decoded);
    }

    // Empty dpc locator should not fail
    #[test]
    fn test_doc_locator_empty() {
        let mut report = ProblemReport::new("Test doc locator empty");
        let locator = DocLocator(vec![]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        locator.encode(&mut encoder, &mut report).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let decoded = DocLocator::decode(&mut decoder, &mut report).unwrap();
        assert_eq!(locator, decoded);
    }
}
