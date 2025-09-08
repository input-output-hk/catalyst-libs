//! `signed_doc.json` headers content type field JSON definition

/// `signed_doc.json` "content type" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct ContentType {
    pub(crate) required: super::IsRequired,
    pub(crate) value: String,
}
