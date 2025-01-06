//! COSE Signature Protected Header `kid` URI Catalyst User Role.

use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

use thiserror::Error;

/// Role Index parsing error
#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RoleIndexError {
    /// Failed to parse the role index
    #[error("Invalid Role Index")]
    InvalidRole(#[from] ParseIntError),
}

/// Project Catalyst User Role Index.
///
/// <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-catalyst-role-definitions/CIP-XXXX/README.md>
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleIndex(u16);

impl From<u16> for RoleIndex {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl FromStr for RoleIndex {
    type Err = RoleIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u16>()?))
    }
}

impl Display for RoleIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}