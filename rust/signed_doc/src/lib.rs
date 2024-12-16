//! Catalyst documents signing crate
#![allow(dead_code)]
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use coset::CborSerializable;

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

impl Display for CatalystSignedDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata: {:?}", self.inner.metadata)?;
        writeln!(f, "JSON Payload: {}", self.inner.payload)?;
        writeln!(f, "Signatures: {:?}", self.inner.signatures)?;
        write!(f, "Content Errors: {:?}", self.content_errors)
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

impl Default for Metadata {
    fn default() -> Self {
        Self {
            r#type: CatalystSignedDocument::INVALID_UUID,
            id: CatalystSignedDocument::INVALID_UUID,
            ver: CatalystSignedDocument::INVALID_UUID,
            r#ref: None,
            template: None,
            reply: None,
            section: None,
        }
    }
}

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
    type Error = anyhow::Error;

    #[allow(clippy::todo)]
    fn try_from(cose_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let cose = coset::CoseSign::from_slice(&cose_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid COSE Sign document: {e}"))?;
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
        let inner = InnerCatalystSignedDocument {
            cose_sign: cose,
            payload,
            ..Default::default()
        };
        Ok(CatalystSignedDocument {
            inner: Arc::new(inner),
            content_errors: Vec::new(),
        })
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

    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> uuid::Uuid {
        self.inner.metadata.r#type
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> uuid::Uuid {
        self.inner.metadata.id
    }
}

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "content encoding";
/// Catalyst Signed Document Content Encoding Value.
const CONTENT_ENCODING_VALUE: &str = "br";
/// CBOR tag for UUID content.
const UUID_CBOR_TAG: u64 = 37;

/// Generate the COSE protected header used by Catalyst Signed Document.
fn cose_protected_header() -> coset::Header {
    coset::HeaderBuilder::new()
        .algorithm(coset::iana::Algorithm::EdDSA)
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
        Ok(DocumentRef::WithVer { id, ver })
    }
}

/// Extract `Metadata` from `coset::CoseSign`.
fn validate_cose_protected_header(cose: &coset::CoseSign) -> anyhow::Result<Metadata> {
    let expected_header = cose_protected_header();
    anyhow::ensure!(
        cose.protected.header.alg == expected_header.alg,
        "Invalid COSE document protected header `algorithm` field"
    );
    anyhow::ensure!(
        cose.protected.header.content_type == expected_header.content_type,
        "Invalid COSE document protected header `content-type` field"
    );
    anyhow::ensure!(
        cose.protected.header.rest.iter().any(|(key, value)| {
            key == &coset::Label::Text(CONTENT_ENCODING_KEY.to_string())
                && value == &coset::cbor::Value::Text(CONTENT_ENCODING_VALUE.to_string())
        }),
        "Invalid COSE document protected header {CONTENT_ENCODING_KEY} field"
    );
    let mut metadata = Metadata::default();

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("type".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `type` field");
    };
    metadata.r#type = decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `type` field, err: {e}"))?;

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("id".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `id` field");
    };
    decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `id` field, err: {e}"))?;

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("ver".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `ver` field");
    };
    decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `ver` field, err: {e}"))?;

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("ref".to_string()))
    {
        decode_cbor_document_ref(value)
            .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `ref` field, err: {e}"))?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("template".to_string()))
    {
        decode_cbor_document_ref(value).map_err(|e| {
            anyhow::anyhow!("Invalid COSE protected header `template` field, err: {e}")
        })?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("reply".to_string()))
    {
        decode_cbor_document_ref(value).map_err(|e| {
            anyhow::anyhow!("Invalid COSE protected header `reply` field, err: {e}")
        })?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("section".to_string()))
    {
        anyhow::ensure!(
            value.is_text(),
            "Invalid COSE protected header, missing `section` field"
        );
    }

    Ok(metadata)
}
