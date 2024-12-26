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
    /// Metadata Content Errors
    #[serde(skip)]
    content_errors: Vec<String>,
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

    /// Are there any validation errors (as opposed to structural errors).
    #[must_use]
    pub fn has_error(&self) -> bool {
        !self.content_errors.is_empty()
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
            content_errors: Vec::new(),
        }
    }
}

/// Errors found when decoding content.
#[derive(Default, Debug)]
struct ContentErrors(Vec<String>);

impl ContentErrors {
    /// Appends an element to the back of the collection
    fn push(&mut self, error_string: String) {
        self.0.push(error_string);
    }
}

impl From<&coset::ProtectedHeader> for Metadata {
    #[allow(clippy::too_many_lines)]
    fn from(protected: &coset::ProtectedHeader) -> Self {
        let mut metadata = Metadata::default();
        let mut errors = Vec::new();

        match cose_protected_header_find(protected, "type") {
            Some(doc_type) => {
                match UuidV4::try_from(&doc_type) {
                    Ok(doc_type_uuid) => {
                        metadata.r#type = doc_type_uuid;
                    },
                    Err(e) => {
                        errors.push(format!("Document `type` is invalid: {e}"));
                    },
                }
            },
            None => errors.push("Invalid COSE protected header, missing `type` field".to_string()),
        };

        match cose_protected_header_find(protected, "id") {
            Some(doc_id) => {
                match UuidV7::try_from(&doc_id) {
                    Ok(doc_id_uuid) => {
                        metadata.id = doc_id_uuid;
                    },
                    Err(e) => {
                        errors.push(format!("Document `id` is invalid: {e}"));
                    },
                }
            },
            None => errors.push("Invalid COSE protected header, missing `id` field".to_string()),
        };

        match cose_protected_header_find(protected, "ver") {
            Some(doc_ver) => {
                match UuidV7::try_from(&doc_ver) {
                    Ok(doc_ver_uuid) => {
                        if doc_ver_uuid < metadata.id {
                            errors.push(format!(
                            "Document Version {doc_ver_uuid} cannot be smaller than Document ID {}", metadata.id
                        ));
                        } else {
                            metadata.ver = doc_ver_uuid;
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

        if let Some(cbor_doc_ref) = cose_protected_header_find(protected, "ref") {
            match DocumentRef::try_from(&cbor_doc_ref) {
                Ok(doc_ref) => {
                    metadata.r#ref = Some(doc_ref);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `ref` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_template) = cose_protected_header_find(protected, "template") {
            match DocumentRef::try_from(&cbor_doc_template) {
                Ok(doc_template) => {
                    metadata.template = Some(doc_template);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `template` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_reply) = cose_protected_header_find(protected, "reply") {
            match DocumentRef::try_from(&cbor_doc_reply) {
                Ok(doc_reply) => {
                    metadata.reply = Some(doc_reply);
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `reply` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_section) = cose_protected_header_find(protected, "section") {
            match cbor_doc_section.into_text() {
                Ok(doc_section) => {
                    metadata.section = Some(doc_section);
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

/// Find a value for a given key in the protected header.
fn cose_protected_header_find(
    protected: &coset::ProtectedHeader, rest_key: &str,
) -> Option<coset::cbor::Value> {
    protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text(rest_key.to_string()))
        .map(|(_, value)| value.clone())
}
