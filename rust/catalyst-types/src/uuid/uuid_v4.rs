//! `UUIDv4` Type.
use std::fmt::{Display, Formatter};

use super::{decode_cbor_uuid, INVALID_UUID};

/// Type representing a `UUIDv4`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "uuid::Uuid")]
pub struct UuidV4 {
    /// UUID
    uuid: uuid::Uuid,
}

impl UuidV4 {
    /// Version for `UUIDv4`.
    const UUID_VERSION_NUMBER: usize = 4;

    /// Generates a zeroed out `UUIDv4` that can never be valid.
    pub fn invalid() -> Self {
        Self { uuid: INVALID_UUID }
    }

    /// Check if this is a valid `UUIDv4`.
    pub fn is_valid(&self) -> bool {
        self.uuid != INVALID_UUID && self.uuid.get_version_num() == Self::UUID_VERSION_NUMBER
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

impl Display for UuidV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.uuid)
    }
}

impl TryFrom<&coset::cbor::Value> for UuidV4 {
    type Error = anyhow::Error;

    fn try_from(cbor_value: &coset::cbor::Value) -> Result<Self, Self::Error> {
        match decode_cbor_uuid(cbor_value) {
            Ok(uuid) => {
                if uuid.get_version_num() == Self::UUID_VERSION_NUMBER {
                    Ok(Self { uuid })
                } else {
                    anyhow::bail!("UUID {uuid} is not `v{}`", Self::UUID_VERSION_NUMBER);
                }
            },
            Err(e) => {
                anyhow::bail!("Invalid UUID. Error: {e}");
            },
        }
    }
}

/// Returns a `UUIDv4` from `uuid::Uuid`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<uuid::Uuid> for UuidV4 {
    fn from(uuid: uuid::Uuid) -> Self {
        Self { uuid }
    }
}