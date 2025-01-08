//! Document Version.
use std::fmt::{Display, Formatter};

use super::UuidV7;

/// Catalyst Document Version.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
pub struct DocumentVersion(UuidV7);

impl DocumentVersion {
    /// Generates a zeroed out `UUIDv7` that can never be valid.
    pub fn invalid() -> Self {
        Self(UuidV7::invalid())
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.0.uuid()
    }
}

impl Display for DocumentVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl From<UuidV7> for DocumentVersion {
    fn from(value: UuidV7) -> Self {
        Self(value)
    }
}
