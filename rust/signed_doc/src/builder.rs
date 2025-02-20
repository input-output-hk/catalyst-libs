//! Catalyst Signed Document Builder.
use catalyst_types::{id_uri::IdUri, problem_report::ProblemReport};
use ed25519_dalek::{ed25519::signature::Signer, SecretKey};

use crate::{
    CatalystSignedDocument, Content, InnerCatalystSignedDocument, Metadata, PROBLEM_REPORT_CTX,
};

/// Catalyst Signed Document Builder.
#[derive(Debug)]
pub struct Builder(InnerCatalystSignedDocument);

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        let report = ProblemReport::new(PROBLEM_REPORT_CTX);
        Self(InnerCatalystSignedDocument {
            report,
            metadata: Default::default(),
            content: Default::default(),
            signatures: Default::default(),
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

    /// Set decoded (original) document content bytes.
    /// Collects a problem report if content is invalid.
    #[must_use]
    pub fn with_decoded_content(mut self, content: Vec<u8>) -> Self {
        let content_type = self.0.metadata.content_type().ok();
        self.0.content = Content::from_decoded(content, content_type, &self.0.report);
        self
    }

    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(mut self, sk: SecretKey, kid: IdUri) -> anyhow::Result<Self> {
        let cose_sign = self
            .0
            .as_cose_sign()
            .map_err(|e| anyhow::anyhow!("Failed to sign: {e}"))?;

        let sk = ed25519_dalek::SigningKey::from_bytes(&sk);
        let protected_header = coset::HeaderBuilder::new().key_id(kid.to_string().into_bytes());

        let mut signature = coset::CoseSignatureBuilder::new()
            .protected(protected_header.build())
            .build();
        let data_to_sign = cose_sign.tbs_data(&[], &signature);
        signature.signature = sk.sign(&data_to_sign).to_vec();
        self.0.signatures.push(kid, signature);

        Ok(self)
    }

    /// Build a signed document with the collected error report.
    /// Could provide an invalid document.
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
