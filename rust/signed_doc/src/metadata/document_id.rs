//! Document ID.
use std::fmt::{Display, Formatter};

use super::UuidV7;

/// Catalyst Document ID.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "UuidV7")]
pub struct DocumentId {
    /// Inner UUID type
    uuid: UuidV7,
}

impl DocumentId {
    /// Generates a zeroed out `UUIDv7` that can never be valid.
    pub fn invalid() -> Self {
        Self {
            uuid: UuidV7::invalid(),
        }
    }

    /// Check if this is a valid `UUIDv7`.
    pub fn is_valid(&self) -> bool {
        self.uuid.is_valid()
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid.uuid()
    }
}

impl Display for DocumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.uuid)
    }
}

impl From<UuidV7> for DocumentId {
    fn from(uuid: UuidV7) -> Self {
        Self { uuid }
    }
}
