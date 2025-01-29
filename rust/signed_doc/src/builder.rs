//! Catalyst Signed Document Builder.
use catalyst_types::kid_uri::KidUri;
use ed25519_dalek::{ed25519::signature::Signer, SecretKey};

use crate::{CatalystSignedDocument, Content, InnerCatalystSignedDocument, Metadata, Signatures};

/// Catalyst Signed Document Builder.
#[derive(Debug, Default, Clone)]
pub struct Builder {
    /// Document Metadata
    pub(crate) metadata: Option<Metadata>,
    /// Document Content
    pub(crate) content: Option<Vec<u8>>,
    /// Signatures
    pub(crate) signatures: Signatures,
}

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set document metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set decoded (original) document content bytes
    #[must_use]
    pub fn with_decoded_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
    }

    /// Set document signatures
    #[must_use]
    pub fn with_signatures(mut self, signatures: Signatures) -> Self {
        self.signatures = signatures;
        self
    }

    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(self, sk: SecretKey, kid: KidUri) -> anyhow::Result<Self> {
        let cose_sign = self
            .clone()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to sign: {e}"))?
            .as_cose_sign()
            .map_err(|e| anyhow::anyhow!("Failed to sign: {e}"))?;
        let Self {
            metadata: Some(metadata),
            content: Some(content),
            mut signatures,
        } = self
        else {
            anyhow::bail!("Metadata and Content are needed for signing");
        };
        let sk = ed25519_dalek::SigningKey::from_bytes(&sk);
        let protected_header = coset::HeaderBuilder::new()
            .key_id(kid.to_string().into_bytes())
            .algorithm(metadata.algorithm().into());
        let mut signature = coset::CoseSignatureBuilder::new()
            .protected(protected_header.build())
            .build();
        let data_to_sign = cose_sign.tbs_data(&[], &signature);
        signature.signature = sk.sign(&data_to_sign).to_vec();
        signatures.push(kid, signature);
        Ok(Self::new()
            .with_decoded_content(content)
            .with_metadata(metadata)
            .with_signatures(signatures))
    }

    /// Build a signed document
    ///
    /// ## Errors
    ///
    /// Fails if any of the fields are missing.
    pub fn build(self) -> anyhow::Result<CatalystSignedDocument> {
        let Some(metadata) = self.metadata else {
            anyhow::bail!("Failed to build Catalyst Signed Document, missing metadata");
        };
        let Some(content) = self.content else {
            anyhow::bail!("Failed to build Catalyst Signed Document, missing document's content");
        };
        let signatures = self.signatures;

        let content = Content::from_decoded(
            content,
            metadata.content_type(),
            metadata.content_encoding(),
        )?;

        Ok(InnerCatalystSignedDocument {
            metadata,
            content,
            signatures,
        }
        .into())
    }
}
