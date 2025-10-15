//! 'updates' field definition

/// Document's 'updates' fields definition.
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Update {
    pub author: bool,
    #[serde(default)]
    pub collaborators: bool,
}
