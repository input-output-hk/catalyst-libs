//! `content-encoding` rule type impl.

use crate::{metadata::ContentEncoding, CatalystSignedDocument};

/// `content-encoding` field validation rule.
#[derive(Debug)]
pub(crate) enum ContentEncodingRule {
    /// Content Encoding field is optionally present in the document.
    Specified {
        /// expected `content-encoding` field.
        exp: ContentEncoding,
        /// optional flag for the `content-encoding` field.
        optional: bool,
    },
    /// Content Encoding field must not be present in the document.
    #[allow(dead_code)]
    NotSpecified,
}

impl ContentEncodingRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        let context = "Content Encoding Rule check";
        match self {
            Self::NotSpecified => {
                if let Some(content_encoding) = doc.doc_content_encoding() {
                    doc.report().unknown_field(
                        "content-encoding",
                        &content_encoding.to_string(),
                        &format!(
                            "{context}, document does not expect to have a content-encoding field"
                        ),
                    );
                    return Ok(false);
                }
            },
            Self::Specified { exp, optional } => {
                if let Some(content_encoding) = doc.doc_content_encoding() {
                    if content_encoding != *exp {
                        doc.report().invalid_value(
                            "content-encoding",
                            content_encoding.to_string().as_str(),
                            exp.to_string().as_str(),
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
                } else if !optional {
                    doc.report().missing_field(
                        "content-encoding",
                        "Document must have a content-encoding field",
                    );
                    return Ok(false);
                }
            },
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[tokio::test]
    async fn content_encoding_is_specified_rule_test() {
        let content_encoding = ContentEncoding::Brotli;

        let rule = ContentEncodingRule::Specified {
            exp: content_encoding,
            optional: true,
        };

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .with_content(content_encoding.encode(&[1, 2, 3]).unwrap())
            .build();
        assert!(rule.check(&doc).await.unwrap());

        // empty content (empty bytes) could not be brotli decoded
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        assert!(!rule.check(&doc).await.unwrap());

        let doc = Builder::new().build();
        assert!(rule.check(&doc).await.unwrap());

        let rule = ContentEncodingRule::Specified {
            exp: content_encoding,
            optional: false,
        };
        assert!(!rule.check(&doc).await.unwrap());
    }

    #[tokio::test]
    async fn content_encoding_is_not_specified_rule_test() {
        let content_encoding = ContentEncoding::Brotli;

        let rule = ContentEncodingRule::NotSpecified;

        // With Brotli content encoding
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        assert!(!rule.check(&doc).await.unwrap());

        // No content encoding
        let doc = Builder::new().build();
        assert!(rule.check(&doc).await.unwrap());
    }
}
