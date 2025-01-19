//! Catalyst Signed Document Metadata.
use coset::cbor::Value;

use super::{decode_cbor_uuid, encode_cbor_value, UuidV7};

/// Reference to a Document.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DocumentRef {
    /// Reference to the latest document
    Latest {
        /// Document ID UUID
        id: UuidV7,
    },
    /// Reference to the specific document version
    WithVer {
        /// Document ID UUID,
        id: UuidV7,
        /// Document Ver UUID
        ver: UuidV7,
    },
}

impl TryFrom<&DocumentRef> for Value {
    type Error = anyhow::Error;

    fn try_from(value: &DocumentRef) -> Result<Self, Self::Error> {
        match value {
            DocumentRef::Latest { id } => encode_cbor_value(id),
            DocumentRef::WithVer { id, ver } => {
                Ok(Value::Array(vec![
                    encode_cbor_value(id)?,
                    encode_cbor_value(ver)?,
                ]))
            },
        }
    }
}

impl TryFrom<&Value> for DocumentRef {
    type Error = anyhow::Error;

    #[allow(clippy::indexing_slicing)]
    fn try_from(val: &Value) -> anyhow::Result<DocumentRef> {
        if let Ok(id) = decode_cbor_uuid(val.clone()) {
            Ok(DocumentRef::Latest { id })
        } else {
            let Some(array) = val.as_array() else {
                anyhow::bail!("Document Reference must be either a single UUID or an array of two");
            };
            anyhow::ensure!(
                array.len() == 2,
                "Document Reference array of two UUIDs was expected"
            );
            let id = decode_cbor_uuid(array[0].clone())?;
            let ver = decode_cbor_uuid(array[1].clone())?;
            anyhow::ensure!(
                ver >= id,
                "Document Reference Version can never be smaller than its ID"
            );
            Ok(DocumentRef::WithVer { id, ver })
        }
    }
}
