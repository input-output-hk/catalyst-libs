//! Catalyst documents signing crate

mod builder;
mod content;
mod decode_context;
pub mod doc_types;
mod metadata;
pub mod providers;
mod signature;
pub mod validator;

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

pub use builder::Builder;
pub use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{Uuid, UuidV4, UuidV7},
};
use cbork_utils::with_cbor_bytes::WithCborBytes;
pub use content::Content;
use decode_context::{CompatibilityPolicy, DecodeContext};
pub use metadata::{
    ContentEncoding, ContentType, DocLocator, DocType, DocumentRef, DocumentRefs, Metadata, Section,
};
use minicbor::{decode, encode, Decode, Decoder, Encode};
pub use signature::{CatalystId, Signatures};

use crate::builder::SignaturesBuilder;

/// `COSE_Sign` object CBOR tag <https://datatracker.ietf.org/doc/html/rfc8152#page-8>
const COSE_SIGN_CBOR_TAG: minicbor::data::Tag = minicbor::data::Tag::new(98);

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Debug)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: WithCborBytes<Metadata>,
    /// Document Content
    content: WithCborBytes<Content>,
    /// Signatures
    signatures: Signatures,
    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    report: ProblemReport,
}

/// Keep all the contents private.
/// Better even to use a structure like this.  Wrapping in an Arc means we don't have to
/// manage the Arc anywhere else. These are likely to be large, best to have the Arc be
/// non-optional.
#[derive(Debug, Clone)]
pub struct CatalystSignedDocument {
    /// Catalyst Signed Document metadata, raw doc, with content errors.
    inner: Arc<InnerCatalystSignedDocument>,
}

impl Display for CatalystSignedDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.inner.metadata.fmt(f)?;
        writeln!(f, "Payload Size: {} bytes", self.inner.content.size())?;
        writeln!(f, "Signature Information")?;
        if self.inner.signatures.is_empty() {
            writeln!(f, "  This document is unsigned.")?;
        } else {
            for kid in &self.kids() {
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

    /// Return Document Type `DocType` - List of `UUIDv4`.
    ///
    /// # Errors
    /// - Missing 'type' field.
    pub fn doc_type(&self) -> anyhow::Result<&DocType> {
        self.inner.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'id' field.
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.inner.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'ver' field.
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.inner.metadata.doc_ver()
    }

    /// Return document content object.
    #[must_use]
    pub(crate) fn content(&self) -> &WithCborBytes<Content> {
        &self.inner.content
    }

    /// Return document decoded (original/non compressed) content bytes.
    ///
    /// # Errors
    ///  - Decompression failure
    pub fn decoded_content(&self) -> anyhow::Result<Vec<u8>> {
        if let Some(encoding) = self.doc_content_encoding() {
            encoding.decode(self.encoded_content())
        } else {
            Ok(self.encoded_content().to_vec())
        }
    }

    /// Return document encoded (compressed) content bytes.
    #[must_use]
    pub fn encoded_content(&self) -> &[u8] {
        self.content().bytes()
    }

    /// Return document `ContentType`.
    ///
    /// # Errors
    /// - Missing 'content-type' field.
    pub fn doc_content_type(&self) -> anyhow::Result<ContentType> {
        self.inner.metadata.content_type()
    }

    /// Return document `ContentEncoding`.
    #[must_use]
    pub fn doc_content_encoding(&self) -> Option<ContentEncoding> {
        self.inner.metadata.content_encoding()
    }

    /// Return document metadata content.
    // TODO: remove this and provide getters from metadata like the rest of its fields have.
    #[must_use]
    pub fn doc_meta(&self) -> &WithCborBytes<Metadata> {
        &self.inner.metadata
    }

    /// Return a Document's signatures
    #[must_use]
    pub(crate) fn signatures(&self) -> &Signatures {
        &self.inner.signatures
    }

    /// Return a list of Document's Catalyst IDs.
    #[must_use]
    pub fn kids(&self) -> Vec<CatalystId> {
        self.inner
            .signatures
            .iter()
            .map(|s| s.kid().clone())
            .collect()
    }

    /// Return a list of Document's author IDs (short form of Catalyst IDs).
    #[must_use]
    pub fn authors(&self) -> Vec<CatalystId> {
        self.inner
            .signatures
            .iter()
            .map(|s| s.kid().as_short_id())
            .collect()
    }

    /// Returns a collected problem report for the document.
    /// It accumulates all kind of errors, collected during the decoding, type based
    /// validation and signature verification.
    ///
    /// This is method is only for the public API usage, do not use it internally inside
    /// this crate.
    #[must_use]
    pub fn problem_report(&self) -> ProblemReport {
        self.report().clone()
    }

    /// Returns an internal problem report
    #[must_use]
    pub(crate) fn report(&self) -> &ProblemReport {
        &self.inner.report
    }

    /// Returns a signed document `Builder` pre-loaded with the current signed document's
    /// data.
    ///
    /// # Errors
    ///  - If error returned its probably a bug. `CatalystSignedDocument` must be a valid
    ///    COSE structure.
    pub fn into_builder(&self) -> anyhow::Result<SignaturesBuilder> {
        self.try_into()
    }
}

impl Decode<'_, ()> for CatalystSignedDocument {
    fn decode(d: &mut Decoder<'_>, _ctx: &mut ()) -> Result<Self, decode::Error> {
        let mut report = ProblemReport::new("Catalyst Signed Document Decoding");
        let mut ctx = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report,
        };

        let p = d.position();
        if let Ok(tag) = d.tag() {
            if tag != COSE_SIGN_CBOR_TAG {
                return Err(minicbor::decode::Error::message(format!(
                    "Must be equal to the COSE_Sign tag value: {COSE_SIGN_CBOR_TAG}"
                )));
            }
        } else {
            d.set_position(p);
        }

        if !matches!(d.array()?, Some(4)) {
            return Err(minicbor::decode::Error::message(
                "Must be a definite size array of 4 elements",
            ));
        }

        let metadata_bytes = d.bytes()?;
        let metadata = WithCborBytes::<Metadata>::decode(
            &mut minicbor::Decoder::new(metadata_bytes),
            &mut ctx,
        )?;

        // empty unprotected headers
        let mut map = cbork_utils::map::Map::decode(d, &mut ())?.into_iter();
        if map.next().is_some() {
            ctx.report.unknown_field(
                "unprotected headers",
                "non empty unprotected headers",
                "COSE unprotected headers must be empty",
            );
        }

        let content = WithCborBytes::<Content>::decode(d, &mut ())?;
        let signatures = Signatures::decode(d, &mut ctx)?;

        Ok(InnerCatalystSignedDocument {
            metadata,
            content,
            signatures,
            report,
        }
        .into())
    }
}

impl<C> Encode<C> for CatalystSignedDocument {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut encode::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        // COSE_Sign tag
        // <!https://datatracker.ietf.org/doc/html/rfc8152#page-9>
        e.tag(COSE_SIGN_CBOR_TAG)?;
        e.array(4)?;
        // protected headers (metadata fields)
        e.bytes(
            minicbor::to_vec(&self.inner.metadata)
                .map_err(minicbor::encode::Error::message)?
                .as_slice(),
        )?;
        // empty unprotected headers
        e.map(0)?;
        // content
        e.encode(&self.inner.content)?;
        // signatures
        e.encode(&self.inner.signatures)?;
        Ok(())
    }
}

impl TryFrom<&[u8]> for CatalystSignedDocument {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(minicbor::decode(value)?)
    }
}

impl TryFrom<CatalystSignedDocument> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(value: CatalystSignedDocument) -> Result<Self, Self::Error> {
        Ok(minicbor::to_vec(value)?)
    }
}
