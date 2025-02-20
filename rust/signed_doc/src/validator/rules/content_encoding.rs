//! `content-encoding` rule type impl.

use crate::{metadata::ContentEncoding, CatalystSignedDocument};

/// `content-encoding` field validation rule
pub(crate) struct ContentEncodingRule {
    /// expected `content-encoding` field
    pub(crate) exp: ContentEncoding,
    /// optional flag for the `content-encoding` field
    pub(crate) optional: bool,
}

impl ContentEncodingRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        if let Some(content_encoding) = doc.doc_content_encoding() {
            if content_encoding != self.exp {
                doc.report().invalid_value(
                    "content-encoding",
                    content_encoding.to_string().as_str(),
                    self.exp.to_string().as_str(),
                    "Invalid Document content-encoding value",
                );
                return Ok(false);
            }
        } else if !self.optional {
            doc.report().missing_field(
                "content-encoding",
                "Document must have a content-encoding field",
            );
            return Ok(false);
        }
        Ok(true)
    }
}
