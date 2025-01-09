//! JSON Content
use super::Content;
use crate::metadata::ContentEncoding;

/// JSON encoded content
pub type Json = Content<serde_json::Value>;

impl Default for Json {
    fn default() -> Self {
        serde_json::Value::Object(serde_json::Map::new()).into()
    }
}

impl TryFrom<&[u8]> for Content<serde_json::Value> {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
            .map_err(|e| anyhow::anyhow!("Failed to parse any JSON content: {e}"))
    }
}

impl TryFrom<(&[u8], Option<ContentEncoding>)> for Content<serde_json::Value> {
    type Error = anyhow::Error;

    fn try_from((value, encoding): (&[u8], Option<ContentEncoding>)) -> Result<Self, Self::Error> {
        if let Some(content_encoding) = encoding {
            match content_encoding.decode(&value.to_vec()) {
                Ok(decompressed) => Self::try_from(decompressed.as_slice()),
                Err(e) => {
                    anyhow::bail!("Failed to decode {encoding:?} content: {e}");
                },
            }
        } else {
            Self::try_from(value)
        }
    }
}
