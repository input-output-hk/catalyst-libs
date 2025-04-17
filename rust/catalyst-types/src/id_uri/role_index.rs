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
pub enum RoleIdError {
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
#[non_exhaustive]
pub enum RoleId {
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

impl RoleId {
    /// Is the `RoleId` the default value [`Self::Role0`]
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::Role0
    }
}

impl Default for RoleId {
    fn default() -> Self {
        Self::Role0
    }
}

impl TryFrom<u8> for RoleId {
    type Error = RoleIdError;

    fn try_from(value: u8) -> Result<Self, RoleIdError> {
        match value {
            0 => Ok(Self::Role0),
            1 => Ok(Self::DelegatedRepresentative),
            2 => Ok(Self::VoterDelegation),
            3 => Ok(Self::Proposer),
            _ => Err(RoleIdError::InvalidRole(value)),
        }
    }
}

impl FromStr for RoleId {
    type Err = RoleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;

        Self::try_from(value)
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", *self as u8)
    }
}

impl<'a, C> minicbor::Decode<'a, C> for RoleId {
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let v = <u8 as minicbor::Decode<'a, C>>::decode(d, ctx)?;

        RoleId::try_from(v).map_err(|_| {
            minicbor::decode::Error::message(format!("Unknown role found: RoleId({v})"))
        })
    }
}
