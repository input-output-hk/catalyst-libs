//! Catalyst Signed Document Metadata.

use std::fmt::Display;

use catalyst_types::uuid::CborContext;
use cbork_utils::decode_helper;
use coset::cbor::Value;

use super::{utils::CborUuidV7, UuidV7};

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
            decode_helper::decode_to_end_helper(d, "document reference", ctx).map_err(|err| {
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

impl TryFrom<DocumentRef> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentRef) -> Result<Self, Self::Error> {
        Ok(Value::Array(vec![
            Value::try_from(CborUuidV7(value.id))?,
            Value::try_from(CborUuidV7(value.ver))?,
        ]))
    }
}

impl TryFrom<&Value> for DocumentRef {
    type Error = anyhow::Error;

    #[allow(clippy::indexing_slicing)]
    fn try_from(val: &Value) -> anyhow::Result<DocumentRef> {
        let Some(array) = val.as_array() else {
            anyhow::bail!("Document Reference must be either a single UUID or an array of two");
        };
        anyhow::ensure!(
            array.len() == 2,
            "Document Reference array of two UUIDs was expected"
        );
        let CborUuidV7(id) = CborUuidV7::try_from(&array[0])?;
        let CborUuidV7(ver) = CborUuidV7::try_from(&array[1])?;
        anyhow::ensure!(
            ver >= id,
            "Document Reference Version can never be smaller than its ID"
        );
        Ok(DocumentRef { id, ver })
    }
}
