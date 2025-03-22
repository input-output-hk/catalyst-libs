//! A role number for `RoleData` in RBAC metadata.

/// A role number for `RoleData` in `Cip509RbacMetadata`.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct RoleNumber(u8);

impl RoleNumber {
    /// A number of the `Role0` role.
    pub const ROLE_0: Self = RoleNumber(0);
}

impl From<u8> for RoleNumber {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<RoleNumber> for u8 {
    fn from(value: RoleNumber) -> Self {
        value.0
    }
}
