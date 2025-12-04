//! 'roles' field definition

/// Document's 'roles' fields definition
#[derive(serde::Deserialize)]
pub struct Roles {
    #[serde(default)]
    /// A list of user roles that can post this document
    /// Empty list == No user role can post.
    pub user: Vec<UserRole>,
    #[serde(default)]
    /// A list of admin roles that can post this document
    /// Empty list == No admin role can post.
    /// Placeholder for future use.
    /// Assume that any Admin Role is equivalent, so admin NOT empty means
    /// Must be signed with the temporary admin key.
    pub admin: Vec<AdminRole>,
}

/// Role definition
#[derive(serde::Deserialize)]
pub enum UserRole {
    /// Role 0 - A registered User / Voter - Base Role
    Registered,
    /// Registered for posting proposals
    Proposer,
    /// Registered as a rep for voting purposes.
    Representative,
}

/// Admin Role definitions.
/// Placeholder for future use.
/// Assume that any Admin Role is equivalent,
/// so admin NOT empty means
/// Must be signed with the temporary admin key.
#[derive(serde::Deserialize)]
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
    /// Bulletin Board Operator role.
    #[serde(rename = "Bulletin Board Operator")]
    BulletinBoardOperator,
}
