//! Catalyst Signed Document Metadata.

use std::fmt::Display;

use coset::cbor::Value;

use super::{utils::CborUuidV7, UuidV7};

/// Reference to a Document.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRef {
    /// Reference to the Document Id
    pub id: UuidV7,
    /// Reference to the Document Ver, if not specified the latest document is meant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ver: Option<UuidV7>,
}

impl Display for DocumentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ver) = self.ver {
            write!(f, "id: {}, ver: {}", self.id, ver)
        } else {
            write!(f, "id: {}", self.id)
        }
    }
}

impl TryFrom<DocumentRef> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentRef) -> Result<Self, Self::Error> {
        if let Some(ver) = value.ver {
            Ok(Value::Array(vec![
                Value::try_from(CborUuidV7(value.id))?,
                Value::try_from(CborUuidV7(ver))?,
            ]))
        } else {
            Value::try_from(CborUuidV7(value.id))
        }
    }
}

impl TryFrom<&Value> for DocumentRef {
    type Error = anyhow::Error;

    #[allow(clippy::indexing_slicing)]
    fn try_from(val: &Value) -> anyhow::Result<DocumentRef> {
        if let Ok(CborUuidV7(id)) = CborUuidV7::try_from(val) {
            Ok(DocumentRef { id, ver: None })
        } else {
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
            Ok(DocumentRef { id, ver: Some(ver) })
        }
    }
}
