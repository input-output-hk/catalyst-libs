//! `content-type` rule type impl.

use crate::{metadata::ContentType, CatalystSignedDocument};

/// `content-type` field validation rule
#[derive(Debug)]
pub(crate) enum ContentTypeRule {
    /// Content Type field must be present with the specific type in the document.
    Specified {
        /// expected `content-type` field
        exp: ContentType,
    },
    /// Content Type field must not be present in the document.
    NotSpecified,
}

impl ContentTypeRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        if let Self::NotSpecified = &self {
            if let Some(content_type) = doc.doc_content_type() {
                doc.report().unknown_field(
                    "content-type",
                    content_type.to_string().as_str(),
                    "document does not expect to have the content type field",
                );
                return Ok(false);
            }
        }
        if let Self::Specified { exp } = &self {
            let Some(content_type) = doc.doc_content_type() else {
                doc.report().missing_field(
                    "content-type",
                    "Cannot get a content type field during the field validation",
                );
                return Ok(false);
            };

            if content_type != *exp {
                doc.report().invalid_value(
                    "content-type",
                    content_type.to_string().as_str(),
                    exp.to_string().as_str(),
                    "Invalid Document content-type value",
                );
                return Ok(false);
            }
            let Ok(content) = doc.decoded_content() else {
                doc.report().functional_validation(
                    "Invalid Document content, cannot get decoded bytes",
                    "Cannot get a document content during the content type field validation",
                );
                return Ok(false);
            };
            if self.validate(&content).is_err() {
                doc.report().invalid_value(
                    "payload",
                    &hex::encode(content),
                    &format!("Invalid Document content, should {content_type} encodable"),
                    "Invalid Document content",
                );
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Validates the provided `content` bytes to be a defined `ContentType`.
    fn validate(
        &self,
        content: &[u8],
    ) -> anyhow::Result<()> {
        if let Self::Specified { exp } = self {
            match exp {
                ContentType::Json => {
                    if let Err(e) = serde_json::from_slice::<&serde_json::value::RawValue>(content)
                    {
                        anyhow::bail!("Invalid {} content: {e}", exp)
                    }
                },
                ContentType::Cbor => {
                    let mut decoder = minicbor::Decoder::new(content);

                    decoder.skip()?;

                    if decoder.position() != content.len() {
                        anyhow::bail!("Unused bytes remain in the input after decoding")
                    }
                },
                ContentType::Cddl
                | ContentType::JsonSchema
                | ContentType::Css
                | ContentType::CssHandlebars
                | ContentType::Html
                | ContentType::HtmlHandlebars
                | ContentType::Markdown
                | ContentType::MarkdownHandlebars
                | ContentType::Plain
                | ContentType::PlainHandlebars => {
                    // TODO: not implemented yet
                    anyhow::bail!("`{}` is valid but unavailable yet", exp)
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[tokio::test]
    async fn cbor_with_trailing_bytes_test() {
        // valid cbor: {1: 2} but with trailing 0xff
        let mut buf = Vec::new();
        let mut enc = minicbor::Encoder::new(&mut buf);
        enc.map(1).unwrap().u8(1).unwrap().u8(2).unwrap();
        buf.push(0xFF); // extra byte

        let content_type = ContentType::Cbor;
        let cbor_rule = ContentTypeRule::Specified { exp: content_type };

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(buf)
            .build();

        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn malformed_cbor_bytes_test() {
        // 0xa2 means a map with 2 key-value pairs, but we only give 1 key
        let invalid_bytes = &[0xA2, 0x01];

        let content_type = ContentType::Cbor;
        let cbor_rule = ContentTypeRule::Specified { exp: content_type };

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(invalid_bytes.into())
            .build();

        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn content_type_cbor_rule_test() {
        let content_type = ContentType::Cbor;
        let cbor_rule = ContentTypeRule::Specified { exp: content_type };

        // with json bytes
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));

        // with cbor bytes
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(true)));

        // without content
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));

        // with empty content
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn content_type_json_rule_test() {
        let content_type = ContentType::Json;
        let json_rule = ContentTypeRule::Specified {
            exp: ContentType::Json,
        };

        // with json bytes
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(true)));

        // with cbor bytes
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .with_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        // without content
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        // with empty content
        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        let doc = Builder::new().build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn content_type_not_specified_rule_test() {
        let content_type = ContentType::Json;
        let rule = ContentTypeRule::NotSpecified;

        let doc = Builder::new()
            .with_metadata_field(SupportedField::ContentType(content_type))
            .build();
        assert!(!rule.check(&doc).await.unwrap());

        let doc = Builder::new().build();
        assert!(rule.check(&doc).await.unwrap());
    }
}
