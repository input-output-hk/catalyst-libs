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
        if self.validate(content).is_err() {
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

    /// Validates the provided `content` bytes to be a defined `ContentType`.
    fn validate(&self, content: &[u8]) -> anyhow::Result<()> {
        match self.exp {
            ContentType::Json => {
                if let Err(e) = serde_json::from_slice::<&serde_json::value::RawValue>(content) {
                    anyhow::bail!("Invalid {} content: {e}", self.exp)
                }
            },
            ContentType::Cbor => {
                let mut decoder = minicbor::Decoder::new(content);

                decoder.skip()?;

                if decoder.position() != content.len() {
                    anyhow::bail!("Unused bytes remain in the input after decoding")
                }
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Builder;

    #[tokio::test]
    async fn cbor_with_trailing_bytes_test() {
        // valid cbor: {1: 2} but with trailing 0xff
        let mut buf = Vec::new();
        let mut enc = minicbor::Encoder::new(&mut buf);
        enc.map(1).unwrap().u8(1).unwrap().u8(2).unwrap();
        buf.push(0xFF); // extra byte

        let cbor_rule = ContentTypeRule {
            exp: ContentType::Cbor,
        };

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({ "content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(buf)
            .build();

        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn malformed_cbor_bytes_test() {
        // 0xa2 means a map with 2 key-value pairs, but we only give 1 key
        let invalid_bytes = &[0xA2, 0x01];

        let cbor_rule = ContentTypeRule {
            exp: ContentType::Cbor,
        };

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({ "content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(invalid_bytes.into())
            .build();

        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn content_type_cbor_rule_test() {
        let cbor_rule = ContentTypeRule {
            exp: ContentType::Cbor,
        };

        // with json bytes
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));

        // with cbor bytes
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(true)));

        // without content
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));

        // with empty content
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": cbor_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(vec![])
            .build();
        assert!(matches!(cbor_rule.check(&doc).await, Ok(false)));
    }

    #[tokio::test]
    async fn content_type_json_rule_test() {
        let json_rule = ContentTypeRule {
            exp: ContentType::Json,
        };

        // with json bytes
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": json_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(true)));

        // with cbor bytes
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": json_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        // without content
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": json_rule.exp.to_string() }))
            .unwrap()
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        // with empty content
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"content-type": json_rule.exp.to_string() }))
            .unwrap()
            .with_decoded_content(vec![])
            .build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));

        let doc = Builder::new().build();
        assert!(matches!(json_rule.check(&doc).await, Ok(false)));
    }
}
