//! `metadata` field definition

pub mod chain;
pub mod collaborators;
pub mod doc_ref;
pub mod parameters;
pub mod reply;
pub mod revocations;
pub mod template;

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Metadata {
    pub template: template::Template,
    #[serde(rename = "ref")]
    pub doc_ref: doc_ref::Ref,
    pub reply: reply::Reply,
    pub revocations: revocations::Revocations,
    pub parameters: parameters::Parameters,
    pub chain: chain::Chain,
    pub collaborators: collaborators::Collaborators,
}
