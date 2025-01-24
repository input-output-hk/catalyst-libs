//! Catalyst Signed Document Builder.
use crate::{CatalystSignedDocument, Content, InnerCatalystSignedDocument, Metadata, Signatures};

/// Catalyst Signed Document Builder.
#[derive(Debug, Default)]
pub struct Builder {
    /// Document Metadata
    metadata: Option<Metadata>,
    /// Document Content
    content: Option<Vec<u8>>,
    /// Signatures
    signatures: Signatures,
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

    /// Set document content
    #[must_use]
    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
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
