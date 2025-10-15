//! 'signers' field definition

pub mod roles;
pub mod update;

/// Document's 'signers' fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Signers {
    pub roles: roles::Roles,
    pub update: update::Update,
}
