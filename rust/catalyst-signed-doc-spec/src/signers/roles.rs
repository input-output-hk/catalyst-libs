//! 'roles' field definition

/// Document's 'roles' fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Roles {
    pub user: Vec<Role>,
}

/// Role definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum Role {
    /// Role 0 - A registered User / Voter - Base Role
    Registered,
    /// Registered for posting proposals
    Proposer,
    /// Registered as a rep for voting purposes.
    Representative,
}
