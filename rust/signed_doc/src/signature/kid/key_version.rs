//! COSE Signature Protected Header `kid` Role0 Key Version.

use std::fmt::{Display, Formatter};

/// Version of the Role0 Key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVersion(u16);

impl From<u16> for KeyVersion {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Display for KeyVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
