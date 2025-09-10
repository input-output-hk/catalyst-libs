//! `metadata` field definition

pub mod doc_ref;
pub mod template;

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Metadata {
    pub template: template::Template,
    #[serde(rename = "ref")]
    pub doc_ref: doc_ref::Ref,
}
