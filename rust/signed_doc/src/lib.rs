//! Catalyst documents signing crate
use std::{convert::TryFrom, sync::Arc};

/// Keep all the contents private.
/// Better even to use a structure like this.  Wrapping in an Arc means we don't have to
/// manage the Arc anywhere else. These are likely to be large, best to have the Arc be
/// non-optional.
pub struct CatalystSignedDocument {
    /// Catalyst Signed Document metadata, raw doc, with content errors.
    inner: Arc<InnerCatalystSignedDocument>,
    /// Content Errors found when parsing the Document
    content_errors: Vec<String>,
}

/// Inner type that holds the Catalyst Signed Document with parsing errors.
struct InnerCatalystSignedDocument {
    /// Document Metadata
    _metadata: Metadata,
    /// Raw payload
    _raw_doc: Vec<u8>,
}

/// Document Metadata.
#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    /// Document Type `UUIDv7`.
    pub r#type: uuid::Uuid,
    /// Document ID `UUIDv7`.
    pub id: uuid::Uuid,
    /// Document Version `UUIDv7`.
    pub ver: uuid::Uuid,
    /// Reference to the latest document.
    pub r#ref: Option<DocumentRef>,
    /// Reference to the document template.
    pub template: Option<DocumentRef>,
    /// Reference to the document reply.
    pub reply: Option<DocumentRef>,
    /// Reference to the document section.
    pub section: Option<String>,
}

/// Reference to a Document.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum DocumentRef {
    /// Reference to the latest document
    Latest {
        /// Document ID UUID
        id: uuid::Uuid,
    },
    /// Reference to the specific document version
    WithVer {
        /// Document ID UUID
        id: uuid::Uuid,
        /// Document Version UUID
        ver: uuid::Uuid,
    },
}

// Do this instead of `new`  if we are converting a single parameter into a struct/type we
// should use either `From` or `TryFrom` and reserve `new` for cases where we need
// multiple parameters to actually create the type.  This is much more elegant to use this
// way, in code.
impl TryFrom<Vec<u8>> for CatalystSignedDocument {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        todo!();
    }
}

impl CatalystSignedDocument {
    /// Invalid Doc Type UUID
    const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Are there any validation errors (as opposed to structural errors.
    #[must_use]
    pub fn has_error(&self) -> bool {
        !self.content_errors.is_empty()
    }

    /// Return Document Type UUID.
    #[must_use]
    pub fn doc_type(&self) -> uuid::Uuid {
        INVALID_UUID
    } // Can compare it against INVALID_DOC_TYPE to see if its valid or not.
}
