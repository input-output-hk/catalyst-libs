//! 'signers' field definition

pub mod roles;

/// Document's 'signers' fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Signers {
    pub roles: roles::Roles,
}
