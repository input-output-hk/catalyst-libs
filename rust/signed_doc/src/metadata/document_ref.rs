//! Catalyst Signed Document Metadata.
use coset::cbor::Value;

use super::{decode_cbor_uuid, encode_cbor_uuid, UuidV7};

/// Reference to a Document.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRef {
    /// Reference to the Document Id
    pub id: UuidV7,
    /// Reference to the Document Ver, if not specified the latest document is meant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ver: Option<UuidV7>,
}

impl TryFrom<DocumentRef> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentRef) -> Result<Self, Self::Error> {
        if let Some(ver) = value.ver {
            Ok(Value::Array(vec![
                encode_cbor_uuid(value.id)?,
                encode_cbor_uuid(ver)?,
            ]))
        } else {
            encode_cbor_uuid(value.id)
        }
    }
}

impl TryFrom<&Value> for DocumentRef {
    type Error = anyhow::Error;

    #[allow(clippy::indexing_slicing)]
    fn try_from(val: &Value) -> anyhow::Result<DocumentRef> {
        if let Ok(id) = decode_cbor_uuid(val.clone()) {
            Ok(DocumentRef { id, ver: None })
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
            Ok(DocumentRef { id, ver: Some(ver) })
        }
    }
}
