//! COSE Signature Protected Header `kid` URI Catalyst User Role.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// Project Catalyst User Role associated with the signature.
///
/// <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-catalyst-role-definitions/CIP-XXXX/README.md>
#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum Role {
    /// Voter = 0
    Zero,
    /// Delegated Representative = 1
    One,
    /// Voter Delegation = 2
    Two,
    /// Proposer = 3
    Three,
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Role::Zero),
            "1" => Ok(Role::One),
            "2" => Ok(Role::Two),
            "3" => Ok(Role::Three),
            _ => Err(anyhow::anyhow!("Unknown Role: {}", s)),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", *self as u16)
    }
}
