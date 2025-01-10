//! Catalyst documents signing crate
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use anyhow::anyhow;
use content::Content;
use coset::CborSerializable;

mod content;
mod error;
mod metadata;
mod signature;

pub use metadata::{DocumentRef, Metadata, UuidV7};
pub use signature::KidUri;
use signature::Signatures;

/// Inner type that holds the Catalyst Signed Document with parsing errors.
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Content
    content: Content,
    /// Document Author
    author: KidUri,
    /// Signatures
    signatures: Signatures,
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
        writeln!(f, "Signature Information [")?;
        for kid in &self.inner.signatures.kids() {
            writeln!(f, "  {kid}")?;
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

        let mut errors = Vec::new();

        let metadata = Metadata::try_from(&cose_sign.protected).map_or_else(
            |e| {
                errors.extend(e.0);
                None
            },
            Some,
        );
        let signatures = Signatures::try_from(&cose_sign.signatures).map_or_else(
            |e| {
                errors.extend(e.0);
                None
            },
            Some,
        );
        let author = signatures.as_ref().and_then(|s| s.kids().first().cloned());

        if cose_sign.payload.is_none() {
            errors.push(anyhow!("Document Content is missing"));
        }
        if author.is_none() {
            errors.push(anyhow!("Document Author is missing"));
        }

        match (cose_sign.payload, author, metadata, signatures) {
            (Some(payload), Some(author), Some(metadata), Some(signatures)) => {
                let content = Content::new(
                    payload,
                    metadata.content_type(),
                    metadata.content_encoding(),
                )
                .map_err(|e| {
                    errors.push(anyhow!("Invalid Document Content: {e}"));
                    errors
                })?;

                Ok(CatalystSignedDocument {
                    inner: InnerCatalystSignedDocument {
                        metadata,
                        content,
                        author,
                        signatures,
                    }
                    .into(),
                })
            },
            _ => Err(error::Error(errors)),
        }
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

    /// Return document `Content`.
    #[must_use]
    pub fn doc_content(&self) -> &Content {
        &self.inner.content
    }

    /// Return a Document's author
    #[must_use]
    pub fn author(&self) -> KidUri {
        self.inner.author.clone()
    }
}
