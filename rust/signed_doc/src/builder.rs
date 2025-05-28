//! Catalyst Signed Document Builder.

/// An implementation of [`CborMap`].
mod cbor_map;
/// COSE format utils.
mod cose;

use std::convert::Infallible;

use catalyst_types::catalyst_id::CatalystId;
use cbor_map::CborMap;
use cose::{
    encode_cose_sign, make_cose_signature, make_metadata_header, make_signature_header,
    make_tbs_data,
};

pub type EncodeError = minicbor::encode::Error<Infallible>;

/// Catalyst Signed Document Builder.
#[derive(Debug)]
pub struct Builder {
    /// Mapping from encoded keys to encoded values.
    metadata: CborMap,
    /// Encoded document content.
    content: Vec<u8>,
    /// Encoded COSE Signatures.
    signatures: Vec<Vec<u8>>,
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
        // Question: maybe this should be cached (e.g. frozen once filled)?
        let metadata_header = make_metadata_header(&self.metadata);

        let kid_str = kid.to_string().into_bytes();
        let signature_header = make_signature_header(kid_str.as_slice())?;

        let tbs_data = make_tbs_data(&metadata_header, &signature_header, &self.content)?;
        let signature_bytes = sign_fn(tbs_data);

        let signature = make_cose_signature(&signature_header, &signature_bytes)?;
        self.signatures.push(signature);

        Ok(self)
    }

    /// Build a CBOR-encoded signed document with the collected error report.
    /// Could provide an invalid document.
    #[must_use]
    pub fn build<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let metadata_header = make_metadata_header(&self.metadata);
        encode_cose_sign(e, &metadata_header, &self.content, &self.signatures)
    }
}
