//! Catalyst Signed Document Metadata.

/// Reference to a Document.
#[derive(Copy, Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum DocumentRef {
    /// Reference to the latest document
    Latest {
        /// Document ID UUID
        id: uuid::Uuid,
    },
    /// Reference to the specific document version
    /// Document ID UUID, Document Ver UUID
    WithVer(uuid::Uuid, uuid::Uuid),
}
