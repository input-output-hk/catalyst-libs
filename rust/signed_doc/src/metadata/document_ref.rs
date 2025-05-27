//! Catalyst Signed Document Metadata.

use std::fmt::Display;

use catalyst_types::uuid::CborContext;
use cbork_utils::decode_helper;

use super::UuidV7;

/// Reference to a Document.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRef {
    /// Reference to the Document Id
    pub id: UuidV7,
    /// Reference to the Document Ver
    pub ver: UuidV7,
}

impl Display for DocumentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, ver: {}", self.id, self.ver)
    }
}

impl minicbor::Encode<CborContext> for DocumentRef {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut CborContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        [self.id, self.ver].encode(e, ctx)
    }
}

impl minicbor::Decode<'_, CborContext> for DocumentRef {
    fn decode(
        d: &mut minicbor::Decoder, ctx: &mut CborContext,
    ) -> Result<Self, minicbor::decode::Error> {
        let (id, ver): (UuidV7, UuidV7) =
            decode_helper::decode_helper(d, "document reference", ctx).map_err(|err| {
                err.with_message("Document Reference array of two UUIDs was expected")
            })?;
        if ver < id {
            return Err(minicbor::decode::Error::message(
                "Document Reference Version can never be smaller than its ID",
            ));
        }
        Ok(Self { id, ver })
    }
}
