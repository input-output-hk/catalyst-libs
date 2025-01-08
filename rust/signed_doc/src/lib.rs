//! Catalyst documents signing crate
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use coset::{CborSerializable, TaggedCborSerializable};

mod metadata;
mod payload;
mod signature;

pub use metadata::{DocumentRef, Metadata, UuidV7};
use payload::JsonContent;
pub use signature::KidUri;

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Default)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Payload viewed as JSON Content
    payload: JsonContent,
    /// Signatures
    signatures: Vec<coset::CoseSignature>,
    /// Raw COSE Sign data
    cose_sign: coset::CoseSign,
    /// Raw COSE Sign bytes
    cose_bytes: Vec<u8>,
    /// Content Errors found when parsing the Document
    content_errors: Vec<String>,
}

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
        writeln!(f, "{:#?}\n", self.inner.payload)?;
        writeln!(f, "Signature Information [")?;
        for signature in &self.inner.signatures {
            writeln!(
                f,
                "  {} 0x{:#}",
                String::from_utf8_lossy(&signature.protected.header.key_id),
                hex::encode(signature.signature.as_slice())
            )?;
        }
        writeln!(f, "]\n")?;
        writeln!(f, "Content Errors [")?;
        for error in &self.inner.content_errors {
            writeln!(f, "  {error:#}")?;
        }
        writeln!(f, "]")
    }
}

impl TryFrom<Vec<u8>> for CatalystSignedDocument {
    type Error = anyhow::Error;

    fn try_from(cose_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Try reading as a tagged COSE SIGN, otherwise try reading as untagged.
        let cose = coset::CoseSign::from_tagged_slice(&cose_bytes)
            .or(coset::CoseSign::from_slice(&cose_bytes))
            .map_err(|e| anyhow::anyhow!("Invalid COSE Sign document: {e}"))?;

        let mut content_errors = Vec::new();

        let metadata = Metadata::from(&cose.protected);

        if metadata.has_error() {
            content_errors.extend_from_slice(metadata.content_errors());
        }

        let mut payload = JsonContent::default();

        if let Some(bytes) = &cose.payload {
            match JsonContent::try_from((bytes, metadata.content_encoding())) {
                Ok(c) => payload = c,
                Err(e) => {
                    content_errors.push(format!("Invalid Payload: {e}"));
                },
            }
        } else {
            content_errors.push("COSE payload is empty".to_string());
        };

        let signatures = cose.signatures.clone();
        let inner = InnerCatalystSignedDocument {
            metadata,
            payload,
            signatures,
            cose_sign: cose,
            cose_bytes,
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

    /// Return Raw COSE SIGN bytes.
    #[must_use]
    pub fn cose_sign_bytes(&self) -> &[u8] {
        self.inner.cose_bytes.as_ref()
    }
}
