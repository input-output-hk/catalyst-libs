//! Catalyst Signed Document Builder.
use catalyst_types::catalyst_id::CatalystId;

use crate::{
    signature::{tbs_data, Signature},
    CatalystSignedDocument, Content, Metadata, ProblemReport, Signatures,
};

/// Catalyst Signed Document Builder.
#[derive(Debug)]
pub struct Builder {
    /// metadata
    metadata: Metadata,
    /// content
    content: Content,
    /// signatures
    signatures: Signatures,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: Metadata::default(),
            content: Content::default(),
            signatures: Signatures::default(),
        }
    }

    /// Set document metadata in JSON format
    /// Collect problem report if some fields are missing.
    ///
    /// # Errors
    /// - Fails if it is invalid metadata fields JSON object.
    pub fn with_json_metadata(mut self, json: serde_json::Value) -> anyhow::Result<Self> {
        let metadata = serde_json::from_value(json)?;
        self.metadata = Metadata::from_metadata_fields(metadata, &ProblemReport::new(""));
        Ok(self)
    }

    /// Set decoded (original) document content bytes
    ///
    /// # Errors
    ///  - Compression failure
    pub fn with_decoded_content(mut self, decoded: Vec<u8>) -> anyhow::Result<Self> {
        if let Some(encoding) = self.metadata.content_encoding() {
            self.content = encoding.encode(&decoded)?.into();
        } else {
            self.content = decoded.into();
        }
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
    ) -> anyhow::Result<Self> {
        if kid.is_id() {
            anyhow::bail!("Provided kid should be in a uri format, kid: {kid}");
        }
        let data_to_sign = tbs_data(&kid, &self.metadata, &self.content)?;
        let sign_bytes = sign_fn(data_to_sign);
        self.signatures.push(Signature::new(kid, sign_bytes));

        Ok(self)
    }

    /// Build a signed document with the collected error report.
    /// Could provide an invalid document.
    ///
    /// # Panics
    ///  Should not panic
    #[must_use]
    #[allow(
        clippy::unwrap_used,
        reason = "At this point all the data MUST be correctly encodable, and the final prepared bytes MUST be correctly decodable as a CatalystSignedDocument object."
    )]
    pub fn build(self) -> CatalystSignedDocument {
        let mut e = minicbor::Encoder::new(Vec::new());
        // COSE_Sign tag
        // <!https://datatracker.ietf.org/doc/html/rfc8152#page-9>
        e.tag(minicbor::data::Tag::new(98)).unwrap();
        e.array(4).unwrap();
        // protected headers (metadata fields)
        e.bytes(minicbor::to_vec(&self.metadata).unwrap().as_slice())
            .unwrap();
        // empty unprotected headers
        e.map(0).unwrap();
        // content
        e.encode(&self.content).unwrap();
        // signatures
        e.encode(self.signatures).unwrap();

        CatalystSignedDocument::try_from(e.into_writer().as_slice()).unwrap()
    }
}

impl From<&CatalystSignedDocument> for Builder {
    fn from(value: &CatalystSignedDocument) -> Self {
        Self {
            metadata: value.inner.metadata.clone(),
            content: value.inner.content.clone(),
            signatures: value.inner.signatures.clone(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    /// A test version of the builder, which allows to build a not fully valid catalyst
    /// signed document
    pub(crate) struct Builder(super::Builder);

    impl Default for Builder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Builder {
        /// Start building a signed document
        #[must_use]
        pub(crate) fn new() -> Self {
            Self(super::Builder::new())
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
            self.0 = self.0.add_signature(sign_fn, kid)?;
            Ok(self)
        }

        /// Build a signed document with the collected error report.
        /// Could provide an invalid document.
        pub(crate) fn build(self) -> super::CatalystSignedDocument {
            self.0.build()
        }
    }
}
