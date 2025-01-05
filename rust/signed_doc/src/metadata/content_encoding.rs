//! Document Payload Content Encoding.

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";

/// IANA `CoAP` Content Encoding.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum ContentEncoding {
    /// Brotli compression.format.
    #[serde(rename = "br")]
    Brotli,
}

impl TryFrom<&coset::cbor::Value> for ContentEncoding {
    type Error = anyhow::Error;

    #[allow(clippy::todo)]
    fn try_from(val: &coset::cbor::Value) -> anyhow::Result<ContentEncoding> {
        match val.as_text() {
            Some(encoding) => {
                match encoding.to_string().to_lowercase().as_ref() {
                    "br" => Ok(ContentEncoding::Brotli),
                    _ => anyhow::bail!("Unsupported Content Encoding: {encoding}"),
                }
            },
            _ => {
                anyhow::bail!("Expected Content Encoding to be a string");
            },
        }
    }
}
