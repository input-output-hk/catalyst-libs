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

    /// Proposer that enabling creation, collaboration, and submission of proposals.
    Proposer = 3,

    /// A custom role.
    Unknown(u8),
}

impl RoleId {
    /// Is the `RoleId` the default value [`Self::Role0`]
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::Role0
    }

    /// Returns the `u8` representation of the role.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            RoleId::Role0 => 0,
            RoleId::DelegatedRepresentative => 1,
            RoleId::Proposer => 3,
            RoleId::Unknown(b) => b,
        }
    }

    /// Returns `true` if the role belongs to the canonical set of known roles.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(
            self,
            Self::Role0 | Self::DelegatedRepresentative | Self::Proposer
        )
    }
}

impl Default for RoleId {
    fn default() -> Self {
        Self::Role0
    }
}

impl From<u8> for RoleId {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Role0,
            1 => Self::DelegatedRepresentative,
            3 => Self::Proposer,
            b => Self::Unknown(b),
        }
    }
}

impl From<RoleId> for u8 {
    fn from(role: RoleId) -> u8 {
        role.as_u8()
    }
}

impl FromStr for RoleId {
    type Err = RoleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;

        Ok(Self::from(value))
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_u8())
    }
}

impl<'a, C> minicbor::Decode<'a, C> for RoleId {
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        <u8 as minicbor::Decode<'a, C>>::decode(d, ctx).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use minicbor::{Decoder, Encoder};
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[proptest::property_test]
        fn role_encode(i: u16) {
            let mut buffer = vec![0u8; 16];

            let mut encoder = Encoder::new(buffer.as_mut_slice());

            encoder.int(i.into()).unwrap();

            let mut decoder = Decoder::new(buffer.as_slice());
            let i_str = i.to_string();

            if i > u16::from(u8::MAX) {
                assert!(RoleId::from_str(&i_str).is_err());
                assert!(decoder.decode::<RoleId>().is_err());
            } else {
                let i = u8::try_from(i).unwrap();

                let r = RoleId::from(i);
                let r_str = RoleId::from_str(&i_str).unwrap();
                let r_display = r.to_string();
                let r_dec: RoleId = decoder.decode().unwrap();

                assert_eq!(r, r_str);
                assert_eq!(r, r_dec);
                assert_eq!(i_str, r_display);
            }
        }
    }
}
