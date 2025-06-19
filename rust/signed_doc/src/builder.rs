//! Catalyst Signed Document Builder.
use catalyst_types::{catalyst_id::CatalystId, problem_report::ProblemReport};

use crate::{
    signature::{tbs_data, Signature},
    CatalystSignedDocument, Content, InnerCatalystSignedDocument, Metadata, Signatures,
    PROBLEM_REPORT_CTX,
};

/// Catalyst Signed Document Builder.
#[derive(Debug)]
pub struct Builder(InnerCatalystSignedDocument);

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        let report = ProblemReport::new(PROBLEM_REPORT_CTX);
        Self(InnerCatalystSignedDocument {
            report,
            metadata: Metadata::default(),
            content: Content::default(),
            signatures: Signatures::default(),
            raw_bytes: None,
        })
    }

    /// Set document metadata in JSON format
    /// Collect problem report if some fields are missing.
    ///
    /// # Errors
    /// - Fails if it is invalid metadata fields JSON object.
    pub fn with_json_metadata(mut self, json: serde_json::Value) -> anyhow::Result<Self> {
        let metadata = serde_json::from_value(json)?;
        self.0.metadata = Metadata::from_metadata_fields(metadata, &self.0.report);
        Ok(self)
    }

    /// Set decoded (original) document content bytes
    #[must_use]
    pub fn with_decoded_content(mut self, content: Vec<u8>) -> Self {
        self.0.content = Content::from_decoded(content);
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
        mut self, sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>, kid: CatalystId,
    ) -> anyhow::Result<Self> {
        if kid.is_id() {
            anyhow::bail!("Provided kid should be in a uri format, kid: {kid}");
        }
        let data_to_sign = tbs_data(
            &kid,
            &self.0.metadata,
            self.0
                .content
                .encoded_bytes(self.0.metadata.content_encoding())?,
        )?;
        let sign_bytes = sign_fn(data_to_sign);
        self.0.signatures.push(Signature::new(kid, sign_bytes));

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
            raw_bytes: None,
        })
    }
}
