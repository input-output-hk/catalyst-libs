//! Document Type.
use std::fmt::{Display, Formatter};

use super::UuidV4;

/// Catalyst Document Type.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "UuidV4")]
pub struct DocumentType(UuidV4);

impl DocumentType {
    /// Generates a zeroed out `UUIDv4` that can never be valid.
    pub fn invalid() -> Self {
        Self(UuidV4::invalid())
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.0.uuid()
    }
}

impl Display for DocumentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl From<UuidV4> for DocumentType {
    fn from(value: UuidV4) -> Self {
        Self(value)
    }
}
