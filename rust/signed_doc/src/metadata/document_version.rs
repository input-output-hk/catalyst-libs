//! Document Version.
use std::fmt::{Display, Formatter};

use coset::cbor::Value;

use super::{encode_cbor_uuid, UuidV7};

/// Catalyst Document Version.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
pub struct DocumentVersion(UuidV7);

impl DocumentVersion {
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

impl From<DocumentVersion> for UuidV7 {
    fn from(value: DocumentVersion) -> Self {
        value.0
    }
}

impl TryFrom<DocumentVersion> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentVersion) -> Result<Self, Self::Error> {
        encode_cbor_uuid(value.0).map_err(|e| anyhow::anyhow!("{e}"))
    }
}
