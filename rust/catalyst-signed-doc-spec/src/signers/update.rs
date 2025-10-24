//! 'updates' field definition

/// Document's 'updates' fields definition.
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Update {
    pub r#type: UpdatersType,
}

#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
#[serde(rename_all = "lowercase")]
pub enum UpdatersType {
    Collaborators,
    Ref,
    Author,
}
