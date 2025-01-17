//! Catalyst documents signing crate

mod builder;
mod content;
mod error;
mod metadata;
mod signature;

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

use anyhow::anyhow;
pub use builder::Builder;
pub use content::Content;
use coset::{CborSerializable, Header};
pub use metadata::{AdditionalFields, DocumentRef, Metadata, UuidV4, UuidV7};
pub use minicbor::{decode, encode, Decode, Decoder, Encode};
pub use signature::{KidUri, Signatures};

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Debug, Clone)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Content
    content: Content,
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
        writeln!(f, "Payload Size: {} bytes", self.inner.content.len())?;
        writeln!(f, "Signature Information")?;
        if self.inner.signatures.0.is_empty() {
            writeln!(f, "  This document is unsigned.")?;
        } else {
            for kid in &self.inner.signatures.kids() {
                writeln!(f, "  Signature Key ID: {kid}")?;
            }
        }
        Ok(())
    }
}

impl From<InnerCatalystSignedDocument> for CatalystSignedDocument {
    fn from(inner: InnerCatalystSignedDocument) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> UuidV4 {
        self.inner.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> UuidV7 {
        self.inner.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    #[must_use]
    pub fn doc_ver(&self) -> UuidV7 {
        self.inner.metadata.doc_ver()
    }

    /// Return document `Content`.
    #[must_use]
    pub fn doc_content(&self) -> &Content {
        &self.inner.content
    }

    /// Return document metadata content.
    #[must_use]
    pub fn doc_meta(&self) -> &AdditionalFields {
        self.inner.metadata.extra()
    }

    /// Return a Document's signatures
    #[must_use]
    pub fn signatures(&self) -> &Signatures {
        &self.inner.signatures
    }
}

impl Decode<'_, ()> for CatalystSignedDocument {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, decode::Error> {
        let start = d.position();
        d.skip()?;
        let end = d.position();
        let cose_bytes = d
            .input()
            .get(start..end)
            .ok_or(minicbor::decode::Error::end_of_input())?;

        let cose_sign = coset::CoseSign::from_slice(cose_bytes).map_err(|e| {
            minicbor::decode::Error::message(format!("Invalid COSE Sign document: {e}"))
        })?;

        let mut errors = Vec::new();

        let metadata = Metadata::try_from(&cose_sign.protected).map_or_else(
            |e| {
                errors.extend(e.0 .0);
                None
            },
            Some,
        );
        let signatures = Signatures::try_from(&cose_sign.signatures).map_or_else(
            |e| {
                errors.extend(e.0 .0);
                None
            },
            Some,
        );

        if cose_sign.payload.is_none() {
            errors.push(anyhow!("Document Content is missing"));
        }

        match (cose_sign.payload, metadata, signatures) {
            (Some(payload), Some(metadata), Some(signatures)) => {
                let content = Content::new(
                    payload,
                    metadata.content_type(),
                    metadata.content_encoding(),
                )
                .map_err(|e| {
                    errors.push(anyhow!("Invalid Document Content: {e}"));
                    minicbor::decode::Error::message(error::Error::from(errors))
                })?;

                Ok(CatalystSignedDocument {
                    inner: InnerCatalystSignedDocument {
                        metadata,
                        content,
                        signatures,
                    }
                    .into(),
                })
            },
            _ => Err(minicbor::decode::Error::message(error::Error::from(errors))),
        }
    }
}

impl Encode<()> for CatalystSignedDocument {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut encode::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), encode::Error<W::Error>> {
        let protected_header = Header::try_from(&self.inner.metadata).map_err(|e| {
            minicbor::encode::Error::message(format!("Failed to encode Document Metadata: {e}"))
        })?;

        let mut builder = coset::CoseSignBuilder::new()
            .protected(protected_header)
            .payload(self.inner.content.bytes().to_vec());

        for signature in self.signatures().signatures() {
            builder = builder.add_signature(signature);
        }

        let cose_sign = builder.build();

        let cose_bytes = cose_sign.to_vec().map_err(|e| {
            minicbor::encode::Error::message(format!("Failed to encode COSE Sign document: {e}"))
        })?;

        e.writer_mut()
            .write_all(&cose_bytes)
            .map_err(|_| minicbor::encode::Error::message("Failed to encode to CBOR"))
    }
}
