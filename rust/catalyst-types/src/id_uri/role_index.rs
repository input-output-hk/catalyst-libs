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
    /// Primary required role use for voting and commenting.
    Role0 = 0,
    /// Delegated representative (dRep) that vote on behalf of delegators.
    DelegatedRepresentative = 1,
    /// Voter role that delegates voting power to a chosen representative (dRep).
    VoterDelegation = 2,
    /// Proposer that enabling creation, collaboration, and submission of proposals.
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
