//! Catalyst Signed Document Builder.

/// An implementation of [`CborMap`].
mod cbor_map;
/// Signed payload with signatures as described in
/// [section 2 of RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#section-2).
/// Specialized for Catalyst Signed Document (e.g. no support for unprotected headers).
mod cose_sign;
/// COSE protected header as per [RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#autoid-8),
/// but with some fields omitted when unused by Catalyst and some fields specialized for
/// it.
mod protected_header;

use std::convert::Infallible;

use catalyst_types::{catalyst_id::CatalystId, problem_report::ProblemReport};
use cbor_map::CborMap;
use minicbor::{bytes::ByteVec, data::Tag};

use crate::{
    signature::Signature, CatalystSignedDocument, Content, ContentEncoding,
    InnerCatalystSignedDocument, Metadata, PROBLEM_REPORT_CTX,
};

pub type EncodeError = minicbor::encode::Error<Infallible>;

/// Catalyst Signed Document Builder.
#[derive(Debug, Default)]
pub struct Builder {
    metadata: CborMap,
    content: Option<ByteVec>,
    signatures: Vec<(CatalystId, ByteVec)>,
}

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set document field metadata.
    ///
    /// # Errors
    /// - Fails if it the CBOR encoding fails.
    pub fn add_metadata_field<C, K: minicbor::Encode<C>, V: minicbor::Encode<C>>(
        mut self, ctx: &mut C, key: K, v: V,
    ) -> Result<Self, EncodeError> {
        // Ignoring pre-insert existence of the key.
        let _: Option<_> = self.metadata.encode_and_insert(ctx, key, v)?;
        Ok(self)
    }

    /// Set document content bytes (if content is encoded, it should be aligned with the
    /// encoding algorithm from the `content-encoding` field.
    #[must_use]
    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(
        mut self, sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>, kid: &CatalystId,
    ) -> anyhow::Result<Self> {
        let cose_sign = self
            .0
            .as_cose_sign()
            .map_err(|e| anyhow::anyhow!("Failed to sign: {e}"))?;

        let protected_header = coset::HeaderBuilder::new().key_id(kid.to_string().into_bytes());

        let mut signature = coset::CoseSignatureBuilder::new()
            .protected(protected_header.build())
            .build();
        let data_to_sign = cose_sign.tbs_data(&[], &signature);
        signature.signature = sign_fn(data_to_sign);
        if let Some(sign) = Signature::from_cose_sig(signature, &self.0.report) {
            self.0.signatures.push(sign);
        }

        Ok(self)
    }

    /// Build a signed document with the collected error report.
    /// Could provide an invalid document.
    #[must_use]
    pub fn build(self) -> CatalystSignedDocument {
        self.0.into()
    }
}

impl From<&CatalystSignedDocument> for Builder {
    fn from(value: &CatalystSignedDocument) -> Self {
        Self(InnerCatalystSignedDocument {
            metadata: value.inner.metadata.clone(),
            content: value.inner.content.clone(),
            signatures: value.inner.signatures.clone(),
            report: value.inner.report.clone(),
        })
    }
}
