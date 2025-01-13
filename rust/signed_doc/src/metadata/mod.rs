//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod additional_fields;
mod content_encoding;
mod content_type;
mod document_id;
mod document_ref;
mod document_type;
mod document_version;

use additional_fields::AdditionalFields;
use anyhow::anyhow;
pub use catalyst_types::uuid::{V4 as UuidV4, V7 as UuidV7};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
pub use document_id::DocumentId;
pub use document_ref::DocumentRef;
pub use document_type::DocumentType;
pub use document_version::DocumentVersion;

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";

/// Document Metadata.
///
/// These values are extracted from the COSE Sign protected header.
#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    /// Document Type `UUIDv4`.
    #[serde(rename = "type")]
    doc_type: DocumentType,
    /// Document ID `UUIDv7`.
    id: DocumentId,
    /// Document Version `UUIDv7`.
    ver: DocumentVersion,
    /// Document Payload Content Type.
    #[serde(rename = "content-type")]
    content_type: ContentType,
    /// Document Payload Content Encoding.
    #[serde(rename = "content-encoding")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: AdditionalFields,
}

impl Metadata {
    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> uuid::Uuid {
        self.doc_type.uuid()
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

    /// Returns the Document Content Type, if any.
    #[must_use]
    pub fn content_type(&self) -> ContentType {
        self.content_type
    }

    /// Returns the Document Content Encoding, if any.
    #[must_use]
    pub fn content_encoding(&self) -> Option<ContentEncoding> {
        self.content_encoding
    }

    /// Return reference to additional metadata fields.
    #[must_use]
    pub fn extra(&self) -> &AdditionalFields {
        &self.extra
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  type: {},", self.doc_type)?;
        writeln!(f, "  id: {},", self.id)?;
        writeln!(f, "  ver: {},", self.ver)?;
        writeln!(f, "  content_type: {}", self.content_type)?;
        writeln!(f, "  content_encoding: {:?}", self.content_encoding)?;
        writeln!(f, "  additional_fields: {:?},", self.extra)?;
        writeln!(f, "}}")
    }
}

impl TryFrom<&coset::ProtectedHeader> for Metadata {
    type Error = crate::error::Error;

    fn try_from(protected: &coset::ProtectedHeader) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let mut content_type = None;
        if let Some(value) = protected.header.content_type.as_ref() {
            match ContentType::try_from(value) {
                Ok(ct) => content_type = Some(ct),
                Err(e) => errors.push(anyhow!("Invalid Document Content-Type: {e}")),
            }
        } else {
            errors.push(anyhow!(
                "Invalid COSE protected header, missing Content-Type field"
            ));
        }

        let mut content_encoding = None;
        if let Some(value) = cose_protected_header_find(
            protected,
            |key| matches!(key, coset::Label::Text(label) if label.eq_ignore_ascii_case(CONTENT_ENCODING_KEY)),
        ) {
            match ContentEncoding::try_from(value) {
                Ok(ce) => content_encoding = Some(ce),
                Err(e) => errors.push(anyhow!("Invalid Document Content Encoding: {e}")),
            }
        } else {
            errors.push(anyhow!(
                "Invalid COSE protected header, missing Content-Encoding field"
            ));
        }

        let mut doc_type = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("type".to_string())
        }) {
            match UuidV4::try_from(value) {
                Ok(uuid) => doc_type = Some(uuid),
                Err(e) => errors.push(anyhow!("Document `type` is invalid: {e}")),
            }
        } else {
            errors.push(anyhow!(
                "Invalid COSE protected header, missing `type` field"
            ));
        }

        let mut id = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("id".to_string())
        }) {
            match UuidV7::try_from(value) {
                Ok(uuid) => id = Some(uuid),
                Err(e) => errors.push(anyhow!("Document `id` is invalid: {e}")),
            }
        } else {
            errors.push(anyhow!("Invalid COSE protected header, missing `id` field"));
        }

        let mut ver = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("ver".to_string())
        }) {
            match UuidV7::try_from(value) {
                Ok(uuid) => ver = Some(uuid),
                Err(e) => errors.push(anyhow!("Document `ver` is invalid: {e}")),
            }
        } else {
            errors.push(anyhow!(
                "Invalid COSE protected header, missing `ver` field"
            ));
        }

        let extra = AdditionalFields::try_from(protected).map_or_else(
            |e| {
                errors.extend(e.0);
                None
            },
            Some,
        );

        match (content_type, content_encoding, id, doc_type, ver, extra) {
            (
                Some(content_type),
                content_encoding,
                Some(id),
                Some(doc_type),
                Some(ver),
                Some(extra),
            ) => {
                if ver < id {
                    errors.push(anyhow!(
                        "Document Version {ver} cannot be smaller than Document ID {id}",
                    ));
                    return Err(crate::error::Error(errors));
                }

                Ok(Self {
                    doc_type: doc_type.into(),
                    id: id.into(),
                    ver: ver.into(),
                    content_encoding,
                    content_type,
                    extra,
                })
            },
            _ => Err(crate::error::Error(errors)),
        }
    }
}

/// Find a value for a predicate in the protected header.
fn cose_protected_header_find(
    protected: &coset::ProtectedHeader, mut predicate: impl FnMut(&coset::Label) -> bool,
) -> Option<&coset::cbor::Value> {
    protected
        .header
        .rest
        .iter()
        .find(|(key, _)| predicate(key))
        .map(|(_, value)| value)
}
