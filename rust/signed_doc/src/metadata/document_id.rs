//! Document ID.
use std::fmt::{Display, Formatter};

use coset::cbor::Value;

use super::{encode_cbor_uuid, UuidV7};

/// Catalyst Document ID.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "UuidV7")]
pub struct DocumentId(UuidV7);

impl DocumentId {
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

impl From<DocumentId> for UuidV7 {
    fn from(value: DocumentId) -> Self {
        value.0
    }
}

impl TryFrom<DocumentId> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentId) -> Result<Self, Self::Error> {
        encode_cbor_uuid(value.0)
    }
}
