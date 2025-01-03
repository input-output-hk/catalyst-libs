//! Catalyst documents signing crate
#![allow(dead_code)]
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use coset::{CborSerializable, TaggedCborSerializable};

mod metadata;

pub use metadata::{DocumentRef, Metadata, UuidV7};

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";
/// Catalyst Signed Document Content Encoding Value.
const CONTENT_ENCODING_VALUE: &str = "br";

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
        let mut content_errors = Vec::new();
        let expected_header = cose_protected_header();

        if cose.protected.header.content_type != expected_header.content_type {
            content_errors
                .push("Invalid COSE document protected header `content-type` field".to_string());
        }

        if !cose.protected.header.rest.iter().any(|(key, value)| {
            key == &coset::Label::Text(CONTENT_ENCODING_KEY.to_string())
                && value == &coset::cbor::Value::Text(CONTENT_ENCODING_VALUE.to_string())
        }) {
            content_errors.push(
                "Invalid COSE document protected header {CONTENT_ENCODING_KEY} field".to_string(),
            );
        }
        let metadata = Metadata::from(&cose.protected);
        if metadata.has_error() {
            content_errors.extend_from_slice(metadata.content_errors());
        }
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
            content_errors,
        };
        Ok(CatalystSignedDocument {
            inner: Arc::new(inner),
        })
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Are there any validation errors (as opposed to structural errors).
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

/// Find a value for a given key in the protected header.
fn cose_protected_header_find(
    cose: &coset::CoseSign, rest_key: &str,
) -> Option<coset::cbor::Value> {
    cose.protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text(rest_key.to_string()))
        .map(|(_, value)| value.clone())
}
