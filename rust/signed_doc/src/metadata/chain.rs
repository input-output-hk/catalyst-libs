//! Document Payload Chain.
//!
//! ref: <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/metadata/#chain-link>

use std::{fmt::Display, hash::Hash};

use cbork_utils::{array::Array, decode_context::DecodeCtx};

use crate::DocumentRef;

/// Reference to the previous Signed Document in a sequence.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Chain {
    /// The consecutive sequence number of the current document
    /// in the chain.
    /// The very first document in a sequence is numbered `0` and it
    /// *MUST ONLY* increment by one for each successive document in
    /// the sequence.
    ///
    /// The FINAL sequence number is encoded with the current height
    /// sequence value, negated.
    ///
    /// For example the following values for height define a chain
    /// that has 5 documents in the sequence 0-4, the final height
    /// is negated to indicate the end of the chain:
    /// `0, 1, 2, 3, -4`
    ///
    /// No subsequent document can be chained to a sequence that has
    /// a final chain height.
    height: i32,
    /// Reference to a single Signed Document.
    ///
    /// Can be *ONLY* omitted in the very first document in a sequence.
    document_ref: Option<DocumentRef>,
}

impl Chain {
    /// Creates a new `Chain`.
    #[must_use]
    pub fn new(
        height: i32,
        document_ref: Option<DocumentRef>,
    ) -> Self {
        Self {
            height,
            document_ref,
        }
    }

    /// Gets `height`.
    #[must_use]
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Gets `document_ref`.
    #[must_use]
    pub fn document_ref(&self) -> Option<&DocumentRef> {
        self.document_ref.as_ref()
    }
}

impl Display for Chain {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if let Some(document_ref) = &self.document_ref {
            write!(f, "height: {}, document_ref: {}", self.height, document_ref)
        } else {
            write!(f, "height: {}", self.height)
        }
    }
}

impl minicbor::Encode<()> for Chain {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(if self.document_ref.is_some() { 2 } else { 1 })?;
        self.height.encode(e, &mut ())?;
        if let Some(doc_ref) = &self.document_ref {
            doc_ref.encode(e, &mut ())?;
        }
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Chain {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "Chain decoding";

        let arr = Array::decode(d, &mut DecodeCtx::Deterministic)?;

        let Some(height_bytes) = arr.first() else {
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected [height, ? document_ref], found empty array"
            )));
        };

        let height = minicbor::Decoder::new(height_bytes).int()?;
        let height = height.try_into().map_err(minicbor::decode::Error::custom)?;

        let document_ref = arr
            .get(1)
            .map(|bytes| {
                let mut d = minicbor::Decoder::new(bytes);
                DocumentRef::decode(&mut d, &mut ())
            })
            .transpose()?;

        Ok(Self {
            height,
            document_ref,
        })
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::UuidV7;
    use minicbor::{Decode, Decoder, Encode, Encoder};

    use super::*;

    #[test]
    fn test_chain_encode_decode_without_doc_ref() {
        let chain = Chain {
            height: 0,
            document_ref: None,
        };

        let mut buf = Vec::new();
        let mut enc = Encoder::new(&mut buf);
        chain.encode(&mut enc, &mut ()).unwrap();

        let mut dec = Decoder::new(&buf);
        let decoded = Chain::decode(&mut dec, &mut ()).unwrap();

        assert_eq!(decoded, chain);
    }

    #[test]
    fn test_chain_encode_decode_with_doc_ref() {
        let id = UuidV7::new();
        let ver = UuidV7::new();

        // Create a test document to generate a valid CID for DocLocator
        let test_doc = crate::builder::Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": id.to_string(),
                "ver": ver.to_string(),
                "type": "ab7c2428-c353-4331-856e-385b2eb20546",
                "content-type": crate::ContentType::Json,
            }))
            .expect("Should create metadata")
            .with_json_content(&serde_json::json!({"test": "content"}))
            .expect("Should set content")
            .build()
            .expect("Should build document");

        let cid = test_doc.to_cid_v1().expect("Should generate CID");
        let doc_locator = crate::DocLocator::from(cid);

        let chain = Chain {
            height: 3,
            document_ref: Some(DocumentRef::new(id, ver, doc_locator)),
        };

        let mut buf = Vec::new();
        let mut enc = Encoder::new(&mut buf);
        chain.encode(&mut enc, &mut ()).unwrap();

        let mut dec = Decoder::new(&buf);
        let decoded = Chain::decode(&mut dec, &mut ()).unwrap();

        assert_eq!(decoded, chain);
    }
}
