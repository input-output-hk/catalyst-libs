//! Document Type.
use std::fmt::{Display, Formatter};

use coset::cbor::Value;

use super::{encode_cbor_uuid, UuidV4};

/// Catalyst Document Type.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "UuidV4")]
pub struct DocumentType(UuidV4);

impl DocumentType {
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

impl From<DocumentType> for UuidV4 {
    fn from(value: DocumentType) -> Self {
        value.0
    }
}

impl TryFrom<DocumentType> for Value {
    type Error = anyhow::Error;

    fn try_from(value: DocumentType) -> Result<Self, Self::Error> {
        encode_cbor_uuid(value.0)
    }
}
