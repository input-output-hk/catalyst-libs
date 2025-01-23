//! Catalyst Signed Document Content Payload

use crate::metadata::{ContentEncoding, ContentType};

/// Decompressed Document Content type bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Content(Vec<u8>, ContentType);

impl Content {
    /// Creates a new `Content` value,
    /// verifies a Document's content, that it is correctly encoded and it corresponds and
    /// parsed to the specified type
    ///
    /// # Errors
    /// Returns an error if content is not correctly encoded
    pub fn new(
        mut content: Vec<u8>, content_type: ContentType, encoding: Option<ContentEncoding>,
    ) -> anyhow::Result<Self> {
        if let Some(content_encoding) = encoding {
            println!("here");
            content = content_encoding
                .encode(content.as_slice())
                .map_err(|e| anyhow::anyhow!("Failed to decode {encoding:?} content: {e}"))?;
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
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Return content byte size
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if content is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
