//! Catalyst documents signing crate
#![allow(dead_code)]
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use coset::{CborSerializable, TaggedCborSerializable};

mod metadata;

pub use metadata::{DocumentRef, Metadata};

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "content encoding";
/// Catalyst Signed Document Content Encoding Value.
const CONTENT_ENCODING_VALUE: &str = "br";
/// CBOR tag for UUID content.
const UUID_CBOR_TAG: u64 = 37;

/// Collection of Content Errors.
pub struct ContentErrors(Vec<String>);

/// Keep all the contents private.
/// Better even to use a structure like this.  Wrapping in an Arc means we don't have to
/// manage the Arc anywhere else. These are likely to be large, best to have the Arc be
/// non-optional.
pub struct CatalystSignedDocument {
    /// Catalyst Signed Document metadata, raw doc, with content errors.
    inner: Arc<InnerCatalystSignedDocument>,
}

impl Display for CatalystSignedDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{}", self.inner.metadata)?;
        writeln!(f, "JSON Payload {:#}\n", self.inner.payload)?;
        writeln!(f, "Signatures [")?;
        for signature in &self.inner.signatures {
            writeln!(f, "  0x{:#}", hex::encode(signature.signature.as_slice()))?;
        }
        writeln!(f, "]\n")?;
        writeln!(f, "Content Errors [")?;
        for error in &self.inner.content_errors {
            writeln!(f, "  {error:#}")?;
        }
        writeln!(f, "]")
    }
}

#[derive(Default)]
/// Inner type that holds the Catalyst Signed Document with parsing errors.
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// JSON Payload
    payload: serde_json::Value,
    /// Signatures
    signatures: Vec<coset::CoseSignature>,
    /// Raw COSE Sign bytes
    cose_sign: coset::CoseSign,
    /// Content Errors found when parsing the Document
    content_errors: Vec<String>,
}

// Do this instead of `new`  if we are converting a single parameter into a struct/type we
// should use either `From` or `TryFrom` and reserve `new` for cases where we need
// multiple parameters to actually create the type.  This is much more elegant to use this
// way, in code.
impl TryFrom<Vec<u8>> for CatalystSignedDocument {
    type Error = anyhow::Error;

    fn try_from(cose_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Try reading as a tagged COSE SIGN, otherwise try reading as untagged.
        let cose = coset::CoseSign::from_tagged_slice(&cose_bytes)
            .or(coset::CoseSign::from_slice(&cose_bytes))
            .map_err(|e| anyhow::anyhow!("Invalid COSE Sign document: {e}"))?;

        let (metadata, content_errors) = metadata_from_cose_protected_header(&cose);
        let payload = match &cose.payload {
            Some(payload) => {
                let mut buf = Vec::new();
                let mut bytes = payload.as_slice();
                brotli::BrotliDecompress(&mut bytes, &mut buf)?;
                serde_json::from_slice(&buf)?
            },
            None => {
                println!("COSE missing payload field with the JSON content in it");
                serde_json::Value::Object(serde_json::Map::new())
            },
        };
        let signatures = cose.signatures.clone();
        let inner = InnerCatalystSignedDocument {
            metadata,
            payload,
            signatures,
            cose_sign: cose,
            content_errors: content_errors.0,
        };
        Ok(CatalystSignedDocument {
            inner: Arc::new(inner),
        })
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Are there any validation errors (as opposed to structural errors.
    #[must_use]
    pub fn has_error(&self) -> bool {
        !self.inner.content_errors.is_empty()
    }

    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> uuid::Uuid {
        self.inner.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> uuid::Uuid {
        self.inner.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    #[must_use]
    pub fn doc_ver(&self) -> uuid::Uuid {
        self.inner.metadata.doc_ver()
    }

    /// Return Last Document Reference `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_ref(&self) -> Option<DocumentRef> {
        self.inner.metadata.doc_ref()
    }

    /// Return Document Template `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_template(&self) -> Option<DocumentRef> {
        self.inner.metadata.doc_template()
    }

    /// Return Document Reply `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_reply(&self) -> Option<DocumentRef> {
        self.inner.metadata.doc_reply()
    }

    /// Return Document Reply `Option<DocumentRef>`.
    #[must_use]
    pub fn doc_section(&self) -> Option<String> {
        self.inner.metadata.doc_section()
    }
}

/// Generate the COSE protected header used by Catalyst Signed Document.
fn cose_protected_header() -> coset::Header {
    coset::HeaderBuilder::new()
        .content_format(coset::iana::CoapContentFormat::Json)
        .text_value(
            CONTENT_ENCODING_KEY.to_string(),
            CONTENT_ENCODING_VALUE.to_string().into(),
        )
        .build()
}

/// Decode `CBOR` encoded `UUID`.
fn decode_cbor_uuid(val: &coset::cbor::Value) -> anyhow::Result<uuid::Uuid> {
    let Some((UUID_CBOR_TAG, coset::cbor::Value::Bytes(bytes))) = val.as_tag() else {
        anyhow::bail!("Invalid CBOR encoded UUID type");
    };
    let uuid = uuid::Uuid::from_bytes(
        bytes
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid CBOR encoded UUID type, invalid bytes size"))?,
    );
    Ok(uuid)
}

/// Decode `CBOR` encoded `DocumentRef`.
#[allow(clippy::indexing_slicing)]
fn decode_cbor_document_ref(val: &coset::cbor::Value) -> anyhow::Result<DocumentRef> {
    if let Ok(id) = decode_cbor_uuid(val) {
        Ok(DocumentRef::Latest { id })
    } else {
        let Some(array) = val.as_array() else {
            anyhow::bail!("Invalid CBOR encoded document `ref` type");
        };
        anyhow::ensure!(array.len() == 2, "Invalid CBOR encoded document `ref` type");
        let id = decode_cbor_uuid(&array[0])?;
        let ver = decode_cbor_uuid(&array[1])?;
        Ok(DocumentRef::WithVer(id, ver))
    }
}

/// Find a value for a given key in the protected header.
fn cose_protected_header_find(cose: &coset::CoseSign, rest_key: &str) -> Option<ciborium::Value> {
    cose.protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text(rest_key.to_string()))
        .map(|(_, value)| value.clone())
}

/// Extract `Metadata` from `coset::CoseSign`.
#[allow(clippy::too_many_lines)]
fn metadata_from_cose_protected_header(cose: &coset::CoseSign) -> (Metadata, ContentErrors) {
    let expected_header = cose_protected_header();
    let mut errors = Vec::new();

    if cose.protected.header.content_type != expected_header.content_type {
        errors.push("Invalid COSE document protected header `content-type` field".to_string());
    }

    if !cose.protected.header.rest.iter().any(|(key, value)| {
        key == &coset::Label::Text(CONTENT_ENCODING_KEY.to_string())
            && value == &coset::cbor::Value::Text(CONTENT_ENCODING_VALUE.to_string())
    }) {
        errors.push(
            "Invalid COSE document protected header {CONTENT_ENCODING_KEY} field".to_string(),
        );
    }
    let mut metadata = Metadata::default();

    match cose_protected_header_find(cose, "type") {
        Some(doc_type) => {
            match decode_cbor_uuid(&doc_type) {
                Ok(doc_type_uuid) => {
                    if doc_type_uuid.get_version_num() == 4 {
                        metadata.r#type = doc_type_uuid;
                    } else {
                        errors.push(format!(
                            "Document type is not a valid UUIDv4: {doc_type_uuid}"
                        ));
                    }
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `type` field, err: {e}"
                    ));
                },
            }
        },
        None => errors.push("Invalid COSE protected header, missing `type` field".to_string()),
    };

    match cose_protected_header_find(cose, "id") {
        Some(doc_id) => {
            match decode_cbor_uuid(&doc_id) {
                Ok(doc_id_uuid) => {
                    if doc_id_uuid.get_version_num() == 7 {
                        metadata.id = doc_id_uuid;
                    } else {
                        errors.push(format!("Document ID is not a valid UUIDv7: {doc_id_uuid}"));
                    }
                },
                Err(e) => {
                    errors.push(format!(
                        "Invalid COSE protected header `id` field, err: {e}"
                    ));
                },
            }
        },
        None => errors.push("Invalid COSE protected header, missing `id` field".to_string()),
    };

    match cose_protected_header_find(cose, "ver") {
        Some(doc_ver) => {
            match decode_cbor_uuid(&doc_ver) {
                Ok(doc_ver_uuid) => {
                    let mut is_valid = true;
                    if doc_ver_uuid.get_version_num() != 7 {
                        errors.push(format!(
                            "Document Version is not a valid UUIDv7: {doc_ver_uuid}"
                        ));
                        is_valid = false;
                    }
                    if doc_ver_uuid < metadata.id {
                        errors.push(format!(
                            "Document Version {doc_ver_uuid} cannot be smaller than Document ID {0}", metadata.id
                        ));
                        is_valid = false;
                    }
                    if is_valid {
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

    if let Some(cbor_doc_ref) = cose_protected_header_find(cose, "ref") {
        match decode_cbor_document_ref(&cbor_doc_ref) {
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

    if let Some(cbor_doc_template) = cose_protected_header_find(cose, "template") {
        match decode_cbor_document_ref(&cbor_doc_template) {
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

    if let Some(cbor_doc_reply) = cose_protected_header_find(cose, "reply") {
        match decode_cbor_document_ref(&cbor_doc_reply) {
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

    if let Some(cbor_doc_section) = cose_protected_header_find(cose, "section") {
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

    (metadata, ContentErrors(errors))
}
