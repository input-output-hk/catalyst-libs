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
            if content_encoding.decode(doc.encoded_content()).is_err() {
                doc.report().invalid_value(
                    "payload",
                    &hex::encode(doc.encoded_content()),
                    &format!(
                        "Document content (payload) must decodable by the set content encoding type: {content_encoding}"
                    ),
                    "Invalid Document content value",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{metadata::SupportedField, Builder};

    #[tokio::test]
    async fn content_encoding_rule_test() {
        let content_encoding = ContentEncoding::Brotli;

        let mut rule = ContentEncodingRule {
            exp: content_encoding,
            optional: true,
        };

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .with_decoded_content(vec![1, 2, 3])
            .unwrap()
            .build();
        assert!(rule.check(&doc).await.unwrap());

        // empty content (empty bytes) could not be brotli decoded
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        assert!(!rule.check(&doc).await.unwrap());

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        assert!(!rule.check(&doc).await.unwrap());

        let doc = Builder::new().build();
        assert!(rule.check(&doc).await.unwrap());

        rule.optional = false;
        assert!(!rule.check(&doc).await.unwrap());
    }
}
