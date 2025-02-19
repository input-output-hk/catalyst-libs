//! `content-type` rule type impl.

use crate::{metadata::ContentType, CatalystSignedDocument};

/// `content-type` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContentTypeRule {
    /// expected `content-type` field
    pub(crate) exp: ContentType,
}

impl ContentTypeRule {
    /// Field validation rule
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        let content_type = doc.doc_content_type()?;
        if content_type != self.exp {
            doc.report().invalid_value(
                "content-type",
                content_type.to_string().as_str(),
                self.exp.to_string().as_str(),
                "Invalid Document content-type value",
            );
            return Ok(false);
        }
        Ok(true)
    }
}
