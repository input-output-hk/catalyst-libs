//! 'roles' field definition

/// Document's 'roles' fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Roles {
    #[serde(default)]
    pub user: Vec<UserRole>,
    #[serde(default)]
    pub admin: Vec<AdminRole>,
}

/// Role definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum UserRole {
    /// Role 0 - A registered User / Voter - Base Role
    Registered,
    /// Registered for posting proposals
    Proposer,
    /// Registered as a rep for voting purposes.
    Representative,
}

#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum AdminRole {
    /// Root Certificate Authority role.
    #[serde(rename = "Root CA")]
    RootCA,
    /// Brand Certificate Authority role.
    #[serde(rename = "Brand CA")]
    BrandCA,
    /// Campaign Certificate Authority role.
    #[serde(rename = "Campaign CA")]
    CampaignCA,
    /// Category Certificate Authority role.
    #[serde(rename = "Category CA")]
    CategoryCA,
    /// Root Admin role.
    #[serde(rename = "Root Admin")]
    RootAdmin,
    /// Brand Admin role.
    #[serde(rename = "Brand Admin")]
    BrandAdmin,
    /// Campaign Admin role.
    #[serde(rename = "Campaign Admin")]
    CampaignAdmin,
    /// Category Admin role.
    #[serde(rename = "Category Admin")]
    CategoryAdmin,
    /// Moderator role.
    #[serde(rename = "Moderator")]
    Moderator,
}
