//! 'headers' field definition

pub mod content_type;

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Headers {
    #[serde(rename = "content type")]
    pub content_type: content_type::ContentType,
}
