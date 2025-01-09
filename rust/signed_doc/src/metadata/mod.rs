//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod content_encoding;
mod content_type;
mod document_id;
mod document_ref;
mod document_type;
mod document_version;

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
    #[serde(default, rename = "content-type")]
    content_type: ContentType,
    /// Document Payload Content Encoding.
    #[serde(default, rename = "content-encoding")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: AdditionalFields,
    /// Metadata Content Errors
    #[serde(skip)]
    content_errors: Vec<String>,
}

/// Additional Metadata Fields.
///
/// These values are extracted from the COSE Sign protected header labels.
#[derive(Default, Debug, serde::Deserialize)]
struct AdditionalFields {
    /// Reference to the latest document.
    #[serde(rename = "ref")]
    doc_ref: Option<DocumentRef>,
    /// Hash of the referenced document bytes.
    ref_hash: Option<Vec<u8>>,
    /// Reference to the document template.
    template: Option<DocumentRef>,
    /// Reference to the document reply.
    reply: Option<DocumentRef>,
    /// Reference to the document section.
    section: Option<String>,
}

impl Metadata {
    /// Are there any validation errors (as opposed to structural errors).
    #[must_use]
    pub fn has_error(&self) -> bool {
        !self.content_errors.is_empty()
    }

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

    /// Return Last Document Reference `Option<Vec<u8>>`.
    #[must_use]
    pub fn doc_ref_hash(&self) -> Option<Vec<u8>> {
        self.extra.ref_hash.clone()
    }

    /// Return Last Document Reference `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_ref(&self) -> Option<DocumentRef> {
        self.extra.doc_ref
    }

    /// Return Document Template `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_template(&self) -> Option<DocumentRef> {
        self.extra.template
    }

    /// Return Document Reply `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_reply(&self) -> Option<DocumentRef> {
        self.extra.reply
    }

    /// Return Document Section `Option<String>`.
    #[must_use]
    pub fn doc_section(&self) -> Option<String> {
        self.extra.section.clone()
    }

    /// List of Content Errors.
    #[must_use]
    pub fn content_errors(&self) -> &Vec<String> {
        &self.content_errors
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

impl Default for Metadata {
    fn default() -> Self {
        Self {
            doc_type: DocumentType::invalid(),
            id: DocumentId::invalid(),
            ver: DocumentVersion::invalid(),
            content_type: ContentType::default(),
            content_encoding: None,
            extra: AdditionalFields::default(),
            content_errors: Vec::new(),
        }
    }
}

impl From<&coset::ProtectedHeader> for Metadata {
    #[allow(clippy::too_many_lines)]
    fn from(protected: &coset::ProtectedHeader) -> Self {
        let mut metadata = Metadata::default();
        let mut errors = Vec::new();

        match protected.header.content_type.as_ref() {
            Some(iana_content_type) => {
                match ContentType::try_from(iana_content_type) {
                    Ok(content_type) => metadata.content_type = content_type,
                    Err(e) => {
                        errors.push(format!("Invalid Document Content-Type: {e}"));
                    },
                }
            },
            None => {
                errors.push(
                    "COSE document protected header `content-type` field is missing".to_string(),
                );
            },
        }

        if let Some(value) = cose_protected_header_find(
            protected,
            |key| matches!(key, coset::Label::Text(label) if label.eq_ignore_ascii_case(CONTENT_ENCODING_KEY)),
        ) {
            match ContentEncoding::try_from(value) {
                Ok(encoding) => {
                    metadata.content_encoding = Some(encoding);
                },
                Err(e) => {
                    errors.push(format!("Invalid Document Content Encoding: {e}"));
                },
            }
        } else {
            errors.push(format!(
                "Invalid COSE document protected header '{CONTENT_ENCODING_KEY}' is missing"
            ));
        }

        if let Some(doc_type) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("type".to_string())
        }) {
            match UuidV4::try_from(doc_type) {
                Ok(doc_type_uuid) => {
                    metadata.doc_type = doc_type_uuid.into();
                },
                Err(e) => {
                    errors.push(format!("Document `type` is invalid: {e}"));
                },
            }
        } else {
            errors.push("Invalid COSE protected header, missing `type` field".to_string());
        }

        match cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("id".to_string())
        }) {
            Some(doc_id) => {
                match UuidV7::try_from(doc_id) {
                    Ok(doc_id_uuid) => {
                        metadata.id = doc_id_uuid.into();
                    },
                    Err(e) => {
                        errors.push(format!("Document `id` is invalid: {e}"));
                    },
                }
            },
            None => errors.push("Invalid COSE protected header, missing `id` field".to_string()),
        };

        match cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("ver".to_string())
        }) {
            Some(doc_ver) => {
                match UuidV7::try_from(doc_ver) {
                    Ok(doc_ver_uuid) => {
                        if doc_ver_uuid.uuid() < metadata.id.uuid() {
                            errors.push(format!(
                            "Document Version {doc_ver_uuid} cannot be smaller than Document ID {}", metadata.id
                        ));
                        } else {
                            metadata.ver = doc_ver_uuid.into();
                        }
                    },
                    Err(e) => {
                        errors.push(format!(
                            "Invalid COSE protected header `ver` field, err: {e}"
                        ));
                    },
                }
            },
            None => errors.push("Invalid COSE protected header, missing `ver` field".to_string()),
        }

        if let Some(cbor_doc_ref) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("ref".to_string())
        }) {
            match DocumentRef::try_from(cbor_doc_ref) {
                Ok(doc_ref) => {
                    metadata.extra.doc_ref = Some(doc_ref);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `ref` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_template) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("template".to_string())
        }) {
            match DocumentRef::try_from(cbor_doc_template) {
                Ok(doc_template) => {
                    metadata.extra.template = Some(doc_template);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `template` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_reply) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("reply".to_string())
        }) {
            match DocumentRef::try_from(cbor_doc_reply) {
                Ok(doc_reply) => {
                    metadata.extra.reply = Some(doc_reply);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `reply` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_section) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("section".to_string())
        }) {
            match cbor_doc_section.clone().into_text() {
                Ok(doc_section) => {
                    metadata.extra.section = Some(doc_section);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `section` field, err: {e:?}"
                    ));
                },
            }
        }
        metadata.content_errors = errors;
        metadata
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
