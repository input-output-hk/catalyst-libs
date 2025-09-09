//! `signed_doc.json` headers content type field JSON definition

/// `signed_doc.json` "content type" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct ContentType {
    pub required: super::IsRequired,
    pub value: Option<String>,
}
