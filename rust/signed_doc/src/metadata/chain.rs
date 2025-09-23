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

        let document_ref = match arr.get(1) {
            Some(bytes) => {
                let mut d = minicbor::Decoder::new(bytes);
                Some(DocumentRef::decode(&mut d, &mut ())?)
            },
            None => None,
        };

        Ok(Self {
            height,
            document_ref,
        })
    }
}
