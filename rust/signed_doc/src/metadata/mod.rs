//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod document_ref;
mod uuid_type;

pub use document_ref::DocumentRef;
pub use uuid_type::{UuidV4, UuidV7};

/// Document Metadata.
#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    /// Document Type `UUIDv4`.
    r#type: UuidV4,
    /// Document ID `UUIDv7`.
    id: UuidV7,
    /// Document Version `UUIDv7`.
    ver: UuidV7,
    /// Reference to the latest document.
    r#ref: Option<DocumentRef>,
    /// Reference to the document template.
    template: Option<DocumentRef>,
    /// Reference to the document reply.
    reply: Option<DocumentRef>,
    /// Reference to the document section.
    section: Option<String>,
}

impl Metadata {
    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> uuid::Uuid {
        self.r#type.uuid()
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> uuid::Uuid {
        self.id.uuid()
    }

    /// Return Document Version `UUIDv7`.
    #[must_use]
    pub fn doc_ver(&self) -> uuid::Uuid {
        self.ver.uuid()
    }

    /// Return Last Document Reference `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_ref(&self) -> Option<DocumentRef> {
        self.r#ref
    }

    /// Return Document Template `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_template(&self) -> Option<DocumentRef> {
        self.template
    }

    /// Return Document Reply `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_reply(&self) -> Option<DocumentRef> {
        self.reply
    }

    /// Return Document Section `Option<String>`.
    #[must_use]
    pub fn doc_section(&self) -> Option<String> {
        self.section.clone()
    }
}
impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  doc_type: {},", self.r#type)?;
        writeln!(f, "  doc_id: {},", self.id)?;
        writeln!(f, "  doc_ver: {},", self.ver)?;
        writeln!(f, "  doc_ref: {:?},", self.r#ref)?;
        writeln!(f, "  doc_template: {:?},", self.template)?;
        writeln!(f, "  doc_reply: {:?},", self.reply)?;
        writeln!(f, "  doc_section: {:?}", self.section)?;
        writeln!(f, "}}")
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            r#type: UuidV4::invalid(),
            id: UuidV7::invalid(),
            ver: UuidV7::invalid(),
            r#ref: None,
            template: None,
            reply: None,
            section: None,
        }
    }
}
