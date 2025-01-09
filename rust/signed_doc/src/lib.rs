//! Catalyst documents signing crate
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use anyhow::anyhow;
use coset::{CborSerializable, CoseSignature};

mod error;
mod metadata;
mod payload;
mod signature;

pub use metadata::{DocumentRef, Metadata, UuidV7};
use payload::JsonContent;
pub use signature::KidUri;
use signature::Signatures;

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Default)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Payload viewed as JSON Content
    payload: JsonContent,
    /// Signatures
    signatures: Signatures,
    /// Raw COSE Sign data
    cose_sign: coset::CoseSign,
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
        for signature in &self.inner.cose_sign.signatures {
            writeln!(
                f,
                "  {} 0x{:#}",
                String::from_utf8_lossy(&signature.protected.header.key_id),
                hex::encode(signature.signature.as_slice())
            )?;
        }
        writeln!(f, "]\n")
    }
}

impl TryFrom<&[u8]> for CatalystSignedDocument {
    type Error = error::Error;

    fn try_from(cose_bytes: &[u8]) -> Result<Self, Self::Error> {
        // Try reading as a tagged COSE SIGN, otherwise try reading as untagged.
        let cose_sign = coset::CoseSign::from_slice(cose_bytes)
            .map_err(|e| vec![anyhow::anyhow!("Invalid COSE Sign document: {e}")])?;

        let metadata = Metadata::try_from(&cose_sign.protected)?;

        let mut content_errors = Vec::new();
        let mut payload = JsonContent::default();

        if let Some(bytes) = &cose_sign.payload {
            match JsonContent::try_from((bytes.as_ref(), metadata.content_encoding())) {
                Ok(c) => payload = c,
                Err(e) => {
                    content_errors.push(anyhow!("Invalid Payload: {e}"));
                },
            }
        } else {
            content_errors.push(anyhow!("COSE payload is empty"));
        };

        let mut signatures = Signatures::default();
        match Signatures::try_from(&cose_sign.signatures) {
            Ok(s) => signatures = s,
            Err(errors) => {
                for e in errors.errors() {
                    content_errors.push(anyhow!("{e}"));
                }
            },
        }

        let inner = InnerCatalystSignedDocument {
            metadata,
            payload,
            signatures,
            cose_sign,
        };

        if !content_errors.is_empty() {
            return Err(error::Error(content_errors));
        }

        Ok(CatalystSignedDocument {
            inner: Arc::new(inner),
        })
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

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
    pub fn cose_sign_bytes(&self) -> Vec<u8> {
        self.inner.cose_sign.clone().to_vec().unwrap_or_default()
    }

    /// Return a list of signature KIDs.
    #[must_use]
    pub fn signature_kids(&self) -> Vec<KidUri> {
        self.inner.signatures.kids()
    }

    /// Return a list of signatures.
    #[must_use]
    pub fn signatures(&self) -> Vec<CoseSignature> {
        self.inner.signatures.signatures()
    }
}
