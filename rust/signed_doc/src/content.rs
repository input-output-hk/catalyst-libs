//! Catalyst Signed Document Content Payload

use crate::metadata::{ContentEncoding, ContentType};

/// Decompressed Document Content type bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Content(Vec<u8>, ContentType);

impl Content {
    /// Creates a new `Content` value,
    /// verifies a Document's content, that it is correctly encoded and it corresponds and
    /// parsed to the specified type
    pub fn new(
        mut content: Vec<u8>, content_type: ContentType, encoding: Option<ContentEncoding>,
    ) -> anyhow::Result<Self> {
        if let Some(content_encoding) = encoding {
            content = content_encoding
                .decode(content.as_slice())
                .map_err(|e| anyhow::anyhow!("Failed to decode {encoding:?} content: {e}"))?;
        }

        match content_type {
            ContentType::Json => {
                serde_json::from_slice::<serde_json::Value>(content.as_slice())?;
            },
            ContentType::Cbor => {
                // TODO impelement a CBOR parsing validation
            },
        }

        Ok(Self(content, content_type))
    }

    /// Return `true` if Document's content type is Json
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self.1, ContentType::Json)
    }

    /// Return `true` if Document's content type is Json
    #[must_use]
    pub fn is_cbor(&self) -> bool {
        matches!(self.1, ContentType::Cbor)
    }

    /// Return content bytes
    pub fn bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}
