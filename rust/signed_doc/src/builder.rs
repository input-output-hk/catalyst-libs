//! Catalyst Signed Document Builder.

/// An implementation of [`CborMap`].
mod cbor_map;
/// COSE protected header as per [RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#autoid-8),
/// but with some fields omitted when unused by Catalyst and some fields specialized for
/// it.
mod cose_header;
/// Signed payload with signatures as described in
/// [section 2 of RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#section-2).
/// Specialized for Catalyst Signed Document (e.g. no support for unprotected headers).
mod cose_sign;

use std::convert::Infallible;

use catalyst_types::catalyst_id::CatalystId;
use cbor_map::CborMap;
use cose_header::{make_metadata_header, make_signature_header};
use cose_sign::make_tbs_data;

use crate::CatalystSignedDocument;

pub type EncodeError = minicbor::encode::Error<Infallible>;

/// Catalyst Signed Document Builder.
#[derive(Debug)]
pub struct Builder {
    metadata: CborMap,
    content: Vec<u8>,
    signatures: Vec<(CatalystId, Vec<u8>)>,
}

impl Builder {
    /// Start building a signed document.
    ///
    /// Sets document content bytes. If content is encoded, it should be aligned with the
    /// encoding algorithm from the `content-encoding` field.
    #[must_use]
    pub fn from_content(content: Vec<u8>) -> Self {
        Self {
            metadata: CborMap::default(),
            content: content.into(),
            signatures: vec![],
        }
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

    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(
        mut self, sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>, kid: CatalystId,
    ) -> Result<Self, EncodeError> {
        let metadata_header = make_metadata_header(&self.metadata);

        let kid_str = kid.to_string().into_bytes();
        let signature_header = make_signature_header(kid_str.as_slice())?;

        // Question: maybe this should be cached?
        let tbs_data = make_tbs_data(&metadata_header, &signature_header, &self.content)?;

        let signature = sign_fn(tbs_data);

        self.signatures.push((kid, signature));

        Ok(self)
    }

    /// Build a signed document with the collected error report.
    /// Could provide an invalid document.
    #[must_use]
    pub fn build(self) -> CatalystSignedDocument {
        self.0.into()
    }
}
