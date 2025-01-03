//! Catalyst Signed Document Metadata.
use super::UuidV7;

/// Reference to a Document.
#[derive(Copy, Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum DocumentRef {
    /// Reference to the latest document
    Latest {
        /// Document ID UUID
        id: UuidV7,
    },
    /// Reference to the specific document version
    /// Document ID UUID, Document Ver UUID
    WithVer(UuidV7, UuidV7),
}

impl TryFrom<&coset::cbor::Value> for DocumentRef {
    type Error = anyhow::Error;

    #[allow(clippy::indexing_slicing)]
    fn try_from(val: &coset::cbor::Value) -> anyhow::Result<DocumentRef> {
        if let Ok(id) = UuidV7::try_from(val) {
            Ok(DocumentRef::Latest { id })
        } else {
            let Some(array) = val.as_array() else {
                anyhow::bail!("Document Reference must be either a single UUID or an array of two");
            };
            anyhow::ensure!(
                array.len() == 2,
                "Document Reference array of two UUIDs was expected"
            );
            let id = UuidV7::try_from(&array[0])?;
            let ver = UuidV7::try_from(&array[1])?;
            anyhow::ensure!(
                ver >= id,
                "Document Reference Version can never be smaller than its ID"
            );
            Ok(DocumentRef::WithVer(id, ver))
        }
    }
}
