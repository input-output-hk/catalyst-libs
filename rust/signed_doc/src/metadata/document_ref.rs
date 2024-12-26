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
