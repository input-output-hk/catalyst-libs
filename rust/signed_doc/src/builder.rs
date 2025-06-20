//! Catalyst Signed Document Builder.
use catalyst_types::catalyst_id::CatalystId;

use crate::{
    signature::{tbs_data, Signature},
    CatalystSignedDocument, Content, ContentType, Metadata, Signatures,
};

/// Catalyst Signed Document Builder.
/// Its a type sage state machine which iterates type safely during different stages of
/// the Catalyst Signed Document build process:
/// Setting Metadata -> Setting Content -> Setting Signatures
pub type Builder = MetadataBuilder;

/// Only `metadata` builder part
pub struct MetadataBuilder(BuilderInner);

/// Only `content` builder part
pub struct ContentBuilder(BuilderInner);

/// Only `Signatures` builder part
pub struct SignaturesBuilder(BuilderInner);

#[derive(Default)]
pub struct BuilderInner {
    /// metadata
    metadata: Metadata,
    /// content
    content: Content,
    /// signatures
    signatures: Signatures,
}

impl MetadataBuilder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        Self(BuilderInner::default())
    }

    /// Set document metadata in JSON format
    /// Collect problem report if some fields are missing.
    ///
    /// # Errors
    /// - Fails if it is invalid metadata fields JSON object.
    pub fn with_json_metadata(mut self, json: serde_json::Value) -> anyhow::Result<ContentBuilder> {
        self.0.metadata = Metadata::from_json(json)?;
        Ok(ContentBuilder(self.0))
    }
}

impl ContentBuilder {
    /// Sets an empty content
    pub fn empty_content(self) -> SignaturesBuilder {
        SignaturesBuilder(self.0)
    }

    /// Set the provided JSON content, applying already set `content-encoding`.
    ///
    /// # Errors
    ///  - Verifies that `content-type` field is set to JSON
    ///  - Cannot serialize provided JSON
    ///  - Compression failure
    pub fn with_json_content(
        mut self, json: &serde_json::Value,
    ) -> anyhow::Result<SignaturesBuilder> {
        anyhow::ensure!(
            self.0.metadata.content_type()? == ContentType::Json,
            "Already set metadata field `content-type` is not JSON value"
        );

        let content = serde_json::to_vec(&json)?;
        if let Some(encoding) = self.0.metadata.content_encoding() {
            self.0.content = encoding.encode(&content)?.into();
        } else {
            self.0.content = content.into();
        }

        Ok(SignaturesBuilder(self.0))
    }
}

impl SignaturesBuilder {
    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(
        mut self, sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>, kid: CatalystId,
    ) -> anyhow::Result<Self> {
        if kid.is_id() {
            anyhow::bail!("Provided kid should be in a uri format, kid: {kid}");
        }
        let data_to_sign = tbs_data(&kid, &self.0.metadata, &self.0.content)?;
        let sign_bytes = sign_fn(data_to_sign);
        self.0.signatures.push(Signature::new(kid, sign_bytes));

        Ok(self)
    }

    /// Builds a document from the set `metadata`, `content` and `signatures`.
    ///
    /// # Errors:
    ///  - CBOR encoding/decoding failures
    pub fn build(self) -> anyhow::Result<CatalystSignedDocument> {
        let doc = build_document(&self.0.metadata, &self.0.content, &self.0.signatures)?;
        Ok(doc)
    }
}

/// Build document from the provided `metadata`, `content` and `signatures`, performs all
/// the decoding validation and collects a problem report.
fn build_document(
    metadata: &Metadata, content: &Content, signatures: &Signatures,
) -> anyhow::Result<CatalystSignedDocument> {
    let mut e = minicbor::Encoder::new(Vec::new());
    // COSE_Sign tag
    // <!https://datatracker.ietf.org/doc/html/rfc8152#page-9>
    e.tag(minicbor::data::Tag::new(98))?;
    e.array(4)?;
    // protected headers (metadata fields)
    e.bytes(minicbor::to_vec(metadata)?.as_slice())?;
    // empty unprotected headers
    e.map(0)?;
    // content
    e.encode(content)?;
    // signatures
    e.encode(signatures)?;
    CatalystSignedDocument::try_from(e.into_writer().as_slice())
}

impl From<&CatalystSignedDocument> for SignaturesBuilder {
    fn from(value: &CatalystSignedDocument) -> Self {
        Self(BuilderInner {
            metadata: value.inner.metadata.clone(),
            content: value.inner.content.clone(),
            signatures: value.inner.signatures.clone(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::builder::SignaturesBuilder;

    /// A test version of the builder, which allows to build a not fully valid catalyst
    /// signed document
    pub(crate) struct Builder(super::BuilderInner);

    impl Default for Builder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Builder {
        /// Start building a signed document
        #[must_use]
        pub(crate) fn new() -> Self {
            Self(super::BuilderInner::default())
        }

        /// Add provided `SupportedField` into the `Metadata`.
        pub(crate) fn with_metadata_field(
            mut self, field: crate::metadata::SupportedField,
        ) -> Self {
            self.0.metadata.add_field(field);
            self
        }

        /// Set the content (COSE payload) to the document builder.
        /// It will set the content as its provided, make sure by yourself that
        /// `content-type` and `content-encoding` fields are aligned with the
        /// provided content bytes.
        pub(crate) fn with_content(mut self, content: Vec<u8>) -> Self {
            self.0.content = content.into();
            self
        }

        /// Add a signature to the document
        pub(crate) fn add_signature(
            mut self, sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>, kid: super::CatalystId,
        ) -> anyhow::Result<Self> {
            self.0 = SignaturesBuilder(self.0).add_signature(sign_fn, kid)?.0;
            Ok(self)
        }

        /// Build a signed document with the collected error report.
        /// Could provide an invalid document.
        pub(crate) fn build(self) -> super::CatalystSignedDocument {
            super::build_document(&self.0.metadata, &self.0.content, &self.0.signatures).unwrap()
        }
    }
}
