//! Document ID.
use std::fmt::{Display, Formatter};

use super::UuidV7;

/// Catalyst Document ID.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "UuidV7")]
pub struct DocumentId(UuidV7);

impl DocumentId {
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

impl Display for DocumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl From<UuidV7> for DocumentId {
    fn from(uuid: UuidV7) -> Self {
        Self(uuid)
    }
}
