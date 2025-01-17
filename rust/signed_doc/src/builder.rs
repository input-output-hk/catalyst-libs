//! Catalyst Signed Document Builder.
use crate::{CatalystSignedDocument, Content, InnerCatalystSignedDocument, Metadata, Signatures};

/// Catalyst Signed Document Builder.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Builder {
    /// Document Metadata
    metadata: Option<Metadata>,
    /// Document Content
    content: Option<Content>,
    /// Signatures
    signatures: Option<Signatures>,
}

impl Builder {
    /// Start building a signed document
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set document metadata
    #[must_use]
    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set document content
    #[must_use]
    pub fn content(mut self, content: Content) -> Self {
        self.content = Some(content);
        self
    }

    /// Set document signatures
    #[must_use]
    pub fn signatures(mut self, signatures: Signatures) -> Self {
        self.signatures = Some(signatures);
        self
    }

    /// Build a signed document
    ///
    /// ## Errors
    ///
    /// Returns
    pub fn build(self) -> anyhow::Result<CatalystSignedDocument> {
        match (self.metadata, self.content, self.signatures) {
            (Some(metadata), Some(content), Some(signatures)) => {
                Ok(CatalystSignedDocument {
                    inner: InnerCatalystSignedDocument {
                        metadata,
                        content,
                        signatures,
                    }
                    .into(),
                })
            },
            _ => Err(anyhow::anyhow!("Failed to build Catalyst Signed Document")),
        }
    }
}
