//! COSE Signature Protected Header `kid` URI Catalyst User Role.

use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

use displaydoc::Display;
use thiserror::Error;

/// Role Index parsing error
#[derive(Display, Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RoleIndexError {
    /// Invalid parse
    Parse(#[from] ParseIntError),

    /// Invalid Role Index
    InvalidRole(u8),
}

/// Project Catalyst User Role Index.
///
/// <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-catalyst-role-definitions/CIP-XXXX/README.md>
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RoleIndex {
    /// Can Vote and Comment on Proposals.
    Role0 = 0,
    /// Allows for voters to give their voting power to the dRep who then votes on their
    /// behalf.
    DelegatedRepresentative = 1,
    /// Assigns a voters voting power to a registered dRep.
    VoterDelegation = 2,
    /// Allows the Propers signing key to also be used as a public encryption key.
    Proposer = 3,
}

impl RoleIndex {
    /// Is the `RoleIndex` the default value [Self::Role0]
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::Role0
    }
}

impl Default for RoleIndex {
    fn default() -> Self {
        Self::Role0
    }
}

impl TryFrom<u8> for RoleIndex {
    type Error = RoleIndexError;

    fn try_from(value: u8) -> Result<Self, RoleIndexError> {
        match value {
            0 => Ok(Self::Role0),
            1 => Ok(Self::DelegatedRepresentative),
            2 => Ok(Self::VoterDelegation),
            3 => Ok(Self::Proposer),
            _ => Err(RoleIndexError::InvalidRole(value)),
        }
    }
}

impl FromStr for RoleIndex {
    type Err = RoleIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;

        Self::try_from(value)
    }
}

impl Display for RoleIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
