//! Catalyst documents signing crate

pub mod builder;
pub mod cid_v1;
mod content;
pub mod decode_context;
pub mod doc_types;
mod metadata;
pub mod providers;
mod signature;
pub mod tests_utils;
pub mod validator;

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

pub use catalyst_types::*;
use cbork_utils::{array::Array, decode_context::DecodeCtx, with_cbor_bytes::WithCborBytes};
pub use cid_v1::{Cid, CidError};
pub use content::Content;
use decode_context::{CompatibilityPolicy, DecodeContext};
pub use metadata::{
    Chain, ContentEncoding, ContentType, DocLocator, DocType, DocumentRef, DocumentRefs, Metadata,
    Section,
};
use minicbor::{Decode, Decoder, Encode, decode, encode};
pub use signature::Signatures;

use crate::{builder::SignaturesBuilder, metadata::SupportedLabel, signature::Signature};

/// `COSE_Sign` object CBOR tag <https://datatracker.ietf.org/doc/html/rfc8152#page-8>
const COSE_SIGN_CBOR_TAG: minicbor::data::Tag = minicbor::data::Tag::new(98);

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Debug)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: WithCborBytes<Metadata>,
    /// Document Content
    content: Content,
    /// Signatures
    signatures: Signatures,
    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    report: problem_report::ProblemReport,
}

/// Catalyst Signed Document type.
/// Represents a general (probable invalid) immutable Catalyst Signed Document object.
/// Even as object holds a huge amount of data, cheap for cloning.
/// As its stated above, the constructed/decoded Catalyst Signed Document object could be
/// invalid, the detailed description of issues could be obtained via `report` method.
#[derive(Debug, Clone)]
pub struct CatalystSignedDocument(Arc<InnerCatalystSignedDocument>);

impl Display for CatalystSignedDocument {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        self.0.metadata.fmt(f)?;
        writeln!(f, "Signature Information")?;
        if self.0.signatures.is_empty() {
            writeln!(f, "  This document is unsigned.")?;
        } else {
            for kid in &self.authors() {
                writeln!(f, "  Author ID: {kid}")?;
            }
        }
        Ok(())
    }
}

impl From<InnerCatalystSignedDocument> for CatalystSignedDocument {
    fn from(inner: InnerCatalystSignedDocument) -> Self {
        Self(inner.into())
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Return Document Type `DocType` - List of `UUIDv4`.
    ///
    /// # Errors
    /// - Missing 'type' field.
    pub fn doc_type(&self) -> anyhow::Result<&DocType> {
        self.0.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'id' field.
    pub fn doc_id(&self) -> anyhow::Result<uuid::UuidV7> {
        self.0.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'ver' field.
    pub fn doc_ver(&self) -> anyhow::Result<uuid::UuidV7> {
        self.0.metadata.doc_ver()
    }

    /// Return document content object.
    #[must_use]
    pub(crate) fn content(&self) -> &Content {
        &self.0.content
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
    #[must_use]
    pub fn doc_content_type(&self) -> Option<ContentType> {
        self.0.metadata.content_type()
    }

    /// Return document `ContentEncoding`.
    #[must_use]
    pub fn doc_content_encoding(&self) -> Option<ContentEncoding> {
        self.0.metadata.content_encoding()
    }

    /// Return document metadata content.
    // TODO: remove this and provide getters from metadata like the rest of its fields have.
    #[must_use]
    pub fn doc_meta(&self) -> &WithCborBytes<Metadata> {
        &self.0.metadata
    }

    /// Return a Document's signatures
    #[must_use]
    pub(crate) fn signatures(&self) -> &Signatures {
        &self.0.signatures
    }

    /// Return a list of Document's Signer's Catalyst IDs,
    #[must_use]
    pub fn authors(&self) -> Vec<catalyst_id::CatalystId> {
        self.0
            .signatures
            .iter()
            .map(Signature::kid)
            .cloned()
            .collect()
    }

    /// Checks if the CBOR body of the signed doc is in the older version format before
    /// v0.04.
    ///
    /// # Errors
    ///
    /// Errors from CBOR decoding.
    pub fn is_deprecated(&self) -> anyhow::Result<bool> {
        let mut e = minicbor::Encoder::new(Vec::new());

        let e = e.encode(self.0.metadata.clone())?;
        let e = e.to_owned().into_writer();

        for entry in cbork_utils::map::Map::decode(
            &mut minicbor::Decoder::new(e.as_slice()),
            &mut cbork_utils::decode_context::DecodeCtx::non_deterministic(),
        )? {
            match minicbor::Decoder::new(&entry.key_bytes).decode::<SupportedLabel>()? {
                SupportedLabel::Template
                | SupportedLabel::Ref
                | SupportedLabel::Reply
                | SupportedLabel::Parameters => {
                    if DocumentRefs::is_deprecated_cbor(&entry.value)? {
                        return Ok(true);
                    }
                },
                _ => {},
            }
        }

        Ok(false)
    }

    /// Returns a collected problem report for the document.
    /// It accumulates all kind of errors, collected during the decoding, type based
    /// validation and signature verification.
    ///
    /// # Note:
    /// Be careful, underlying `ProblemReport` state thread safe and wrapped under the
    /// `Arc`, meaning you could easily change the internal state by non-mutable
    /// reference. Any modifications to the returned object would also affect on the
    /// "validity" of the correct `CatalystSignedDocument` instance.
    #[must_use]
    pub fn report(&self) -> &problem_report::ProblemReport {
        &self.0.report
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

    /// Returns CBOR bytes.
    ///
    /// # Errors
    ///  - `minicbor::encode::Error`
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut e = minicbor::Encoder::new(Vec::new());
        self.encode(&mut e, &mut ())?;
        Ok(e.into_writer())
    }

    /// Build `CatalystSignedDoc` instance from CBOR bytes.
    ///
    /// # Errors
    ///  - `minicbor::decode::Error`
    pub fn from_bytes(
        bytes: &[u8],
        mut policy: CompatibilityPolicy,
    ) -> anyhow::Result<Self> {
        Ok(minicbor::decode_with(bytes, &mut policy)?)
    }

    /// Returns a `DocumentRef` for the current document.
    ///
    /// Generating a CID v1 (Content Identifier version 1) creates an IPFS-compatible
    /// content identifier using:
    /// - CID version 1
    /// - CBOR multicodec (0x51)
    /// - SHA2-256 multihash
    ///
    /// # Errors
    ///  - CBOR serialization failure
    ///  - Multihash construction failure
    ///  - Missing 'id' field.
    ///  - Missing 'ver' field.
    pub fn doc_ref(&self) -> anyhow::Result<DocumentRef> {
        let cid = self.to_cid_v1()?;
        Ok(DocumentRef::new(
            self.doc_id()?,
            self.doc_ver()?,
            DocLocator::from(cid),
        ))
    }

    /// Generate a CID v1 (Content Identifier version 1) for this signed document.
    ///
    /// Creates an IPFS-compatible content identifier using:
    /// - CID version 1
    /// - CBOR multicodec (0x51)
    /// - SHA2-256 multihash
    ///
    /// # Errors
    ///  - CBOR serialization failure
    ///  - Multihash construction failure
    fn to_cid_v1(&self) -> Result<cid_v1::Cid, cid_v1::CidError> {
        let cbor_bytes = self
            .to_bytes()
            .map_err(|e| cid_v1::CidError::Encoding(e.to_string()))?;
        cid_v1::to_cid_v1(&cbor_bytes)
    }
}

impl Decode<'_, CompatibilityPolicy> for CatalystSignedDocument {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut CompatibilityPolicy,
    ) -> Result<Self, decode::Error> {
        let mut ctx = DecodeContext::new(
            *ctx,
            problem_report::ProblemReport::new("Catalyst Signed Document Decoding"),
        );

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

        let arr = Array::decode(d, &mut DecodeCtx::Deterministic)?;

        let signed_doc = match arr.as_slice() {
            [
                metadata_bytes,
                headers_bytes,
                content_bytes,
                signatures_bytes,
            ] => {
                let metadata_bytes = minicbor::Decoder::new(metadata_bytes).bytes()?;
                let metadata = WithCborBytes::<Metadata>::decode(
                    &mut minicbor::Decoder::new(metadata_bytes),
                    &mut ctx,
                )?;

                // empty unprotected headers
                let mut map = cbork_utils::map::Map::decode(
                    &mut minicbor::Decoder::new(headers_bytes.as_slice()),
                    &mut cbork_utils::decode_context::DecodeCtx::Deterministic,
                )?
                .into_iter();
                if map.next().is_some() {
                    ctx.report().unknown_field(
                        "unprotected headers",
                        "non empty unprotected headers",
                        "COSE unprotected headers must be empty",
                    );
                }

                let content = Content::decode(
                    &mut minicbor::Decoder::new(content_bytes.as_slice()),
                    &mut (),
                )?;

                let signatures = Signatures::decode(
                    &mut minicbor::Decoder::new(signatures_bytes.as_slice()),
                    &mut ctx,
                )?;

                InnerCatalystSignedDocument {
                    metadata,
                    content,
                    signatures,
                    report: ctx.into_report(),
                }
            },
            _ => {
                return Err(minicbor::decode::Error::message(
                    "Must be a definite size array of 4 elements",
                ));
            },
        };

        Ok(signed_doc.into())
    }
}

impl<C> Encode<C> for CatalystSignedDocument {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut encode::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        // COSE_Sign tag
        // <!https://datatracker.ietf.org/doc/html/rfc8152#page-9>
        e.tag(COSE_SIGN_CBOR_TAG)?;
        e.array(4)?;
        // protected headers (metadata fields)
        e.bytes(
            minicbor::to_vec(&self.0.metadata)
                .map_err(minicbor::encode::Error::message)?
                .as_slice(),
        )?;
        // empty unprotected headers
        e.map(0)?;
        // content
        e.encode(&self.0.content)?;
        // signatures
        e.encode(&self.0.signatures)?;
        Ok(())
    }
}

impl TryFrom<&[u8]> for CatalystSignedDocument {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value, CompatibilityPolicy::Accept)
    }
}

impl TryFrom<&CatalystSignedDocument> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(value: &CatalystSignedDocument) -> Result<Self, Self::Error> {
        value.to_bytes()
    }
}
