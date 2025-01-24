//! Catalyst Signed Document Content Payload

use crate::metadata::{ContentEncoding, ContentType};

/// Decompressed Document Content type bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    /// Content data bytes
    data: Vec<u8>,
    /// Content type
    content_type: ContentType,
    /// Content encoding
    content_encoding: Option<ContentEncoding>,
}

impl Content {
    /// Creates a new `Content` value, from the encoded data.
    /// verifies a Document's content, that it is correctly encoded and it corresponds and
    /// parsed to the specified type
    ///
    /// # Errors
    /// Returns an error if content is not correctly encoded
    pub(crate) fn from_encoded(
        mut data: Vec<u8>, content_type: ContentType, content_encoding: Option<ContentEncoding>,
    ) -> anyhow::Result<Self> {
        if let Some(encoding) = content_encoding {
            data = encoding
                .decode(&data)
                .map_err(|e| anyhow::anyhow!("Failed to decode {encoding} content: {e}"))?;
        }

        Ok(Self {
            data,
            content_type,
            content_encoding,
        })
    }

    /// Creates a new `Content` value, from the decoded (original) data.
    /// verifies that it corresponds and parsed to the specified type.
    ///
    /// # Errors
    /// Returns an error if content is not correctly encoded
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn from_decoded(
        data: Vec<u8>, content_type: ContentType, content_encoding: Option<ContentEncoding>,
    ) -> anyhow::Result<Self> {
        // TODO add content_type verification
        Ok(Self {
            data,
            content_type,
            content_encoding,
        })
    }

    /// Return `true` if Document's content type is Json
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self.content_type, ContentType::Json)
    }

    /// Return `true` if Document's content type is Json
    #[must_use]
    pub fn is_cbor(&self) -> bool {
        matches!(self.content_type, ContentType::Cbor)
    }

    /// Return an decoded (original) content bytes,
    /// by the corresponding `content_encoding` provided field.
    #[must_use]
    pub fn decoded_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Return an encoded content bytes,
    /// by the corresponding `content_encoding` provided field
    pub(crate) fn encoded_bytes(&self) -> anyhow::Result<Vec<u8>> {
        if let Some(encoding) = self.content_encoding {
            let data = encoding
                .encode(&self.data)
                .map_err(|e| anyhow::anyhow!("Failed to encode {encoding} content: {e}"))?;
            Ok(data)
        } else {
            Ok(self.data.clone())
        }
    }

    /// Return content byte size
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return `true` if content is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
