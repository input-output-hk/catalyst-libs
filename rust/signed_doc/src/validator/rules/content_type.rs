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
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        let Ok(content_type) = doc.doc_content_type() else {
            doc.report().missing_field(
                "content-type",
                "Cannot get a content type field during the field validation",
            );
            return Ok(false);
        };
        if content_type != self.exp {
            doc.report().invalid_value(
                "content-type",
                content_type.to_string().as_str(),
                self.exp.to_string().as_str(),
                "Invalid Document content-type value",
            );
            return Ok(false);
        }
        let Ok(content) = doc.doc_content().decoded_bytes() else {
            doc.report().missing_field(
                "payload",
                "Cannot get a document content during the content type field validation",
            );
            return Ok(false);
        };
        if content_type.validate(content).is_err() {
            doc.report().invalid_value(
                "payload",
                &hex::encode(content),
                &format!("Invalid Document content, should {content_type} encodable"),
                "Invalid Document content",
            );
            return Ok(false);
        }

        Ok(true)
    }
}
