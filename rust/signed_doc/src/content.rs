//! Catalyst Signed Document Content Payload

use catalyst_types::problem_report::ProblemReport;

use crate::metadata::{ContentEncoding, ContentType};

/// Decompressed Document Content type bytes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Content {
    /// Original Decompressed Document's data bytes
    data: Option<Vec<u8>>,
}

impl Content {
    /// Creates a new `Content` value, from the encoded data.
    /// verifies a Document's content, that it is correctly encoded and it corresponds and
    /// parsed to the specified type
    pub(crate) fn from_encoded(
        mut data: Vec<u8>, content_type: Option<ContentType>,
        content_encoding: Option<ContentEncoding>, report: &ProblemReport,
    ) -> Self {
        if let Some(content_encoding) = content_encoding {
            if let Ok(decoded_data) = content_encoding.decode(&data) {
                data = decoded_data;
            } else {
                report.invalid_value(
                    "payload",
                    &hex::encode(&data),
                    &format!("Invalid Document content, should {content_encoding} encodable"),
                    "Invalid Document content type.",
                );
                return Self::default();
            }
        }
        if let Some(content_type) = content_type {
            if content_type.validate(&data).is_err() {
                report.invalid_value(
                    "payload",
                    &hex::encode(&data),
                    &format!("Invalid Document content type, should {content_type} encodable"),
                    "Invalid Document content type.",
                );
                return Self::default();
            }
        }

        Self { data: Some(data) }
    }

    /// Creates a new `Content` value, from the decoded (original) data.
    /// verifies that it corresponds and parsed to the specified type.
    ///
    /// # Errors
    /// Returns an error if content is not correctly encoded
    pub(crate) fn from_decoded(data: Vec<u8>, content_type: ContentType) -> anyhow::Result<Self> {
        content_type.validate(&data)?;
        Ok(Self { data: Some(data) })
    }

    /// Return an decoded (original) content bytes.
    ///
    /// # Errors
    ///  - Missing Document content
    pub fn decoded_bytes(&self) -> anyhow::Result<&[u8]> {
        self.data
            .as_deref()
            .ok_or(anyhow::anyhow!("Missing Document content"))
    }

    /// Return an encoded content bytes,
    /// by the provided `content_encoding` provided field.
    ///
    /// # Errors
    ///  - Missing Document content
    ///  - Failed to encode content.
    pub(crate) fn encoded_bytes(
        &self, content_encoding: ContentEncoding,
    ) -> anyhow::Result<Vec<u8>> {
        let content = self.decoded_bytes()?;
        let data = content_encoding
            .encode(content)
            .map_err(|e| anyhow::anyhow!("Failed to encode {content_encoding} content: {e}"))?;
        Ok(data)
    }

    /// Return content byte size.
    /// If content is empty returns `0`.
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.as_ref().map(Vec::len).unwrap_or_default()
    }
}
