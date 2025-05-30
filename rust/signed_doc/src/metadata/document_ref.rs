//! Catalyst Signed Document Metadata.

use std::fmt::Display;

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

impl<C> minicbor::Encode<C> for DocumentRef {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?
            .encode_with(self.id, &mut catalyst_types::uuid::CborContext::Tagged)?
            .encode_with(self.ver, &mut catalyst_types::uuid::CborContext::Tagged)?;
        Ok(())
    }
}
