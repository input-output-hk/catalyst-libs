//! Catalyst Signed Document Content Payload

use anyhow::Context;
use catalyst_types::problem_report::ProblemReport;

use crate::metadata::ContentEncoding;

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
        mut data: Vec<u8>, content_encoding: Option<ContentEncoding>, report: &ProblemReport,
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
        Self::from_decoded(data)
    }

    /// Creates a new `Content` value, from the decoded (original) data.
    pub(crate) fn from_decoded(data: Vec<u8>) -> Self {
        Self { data: Some(data) }
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
        &self, content_encoding: Option<ContentEncoding>,
    ) -> anyhow::Result<Vec<u8>> {
        let content = self.decoded_bytes()?;
        if let Some(content_encoding) = content_encoding {
            content_encoding
                .encode(content)
                .context(format!("Failed to encode {content_encoding} content"))
        } else {
            Ok(content.to_vec())
        }
    }

    /// Return content byte size.
    /// If content is empty returns `0`.
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.as_ref().map(Vec::len).unwrap_or_default()
    }
}
