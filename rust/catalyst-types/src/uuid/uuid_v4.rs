//! `UUIDv4` Type.
use std::fmt::{Display, Formatter};

use minicbor::{Decode, Decoder, Encode};

use super::{decode_cbor_uuid, encode_cbor_uuid, CborContext, INVALID_UUID};

/// Type representing a `UUIDv4`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Serialize)]
pub struct UuidV4(uuid::Uuid);

impl UuidV4 {
    /// Version for `UUIDv4`.
    const UUID_VERSION_NUMBER: usize = 4;

    /// Generates a random `UUIDv4`.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Generates a zeroed out `UUIDv4` that can never be valid.
    #[must_use]
    pub fn invalid() -> Self {
        Self(INVALID_UUID)
    }

    /// Check if this is a valid `UUIDv4`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        is_valid(&self.uuid())
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.0
    }
}

/// Check if this is a valid `UUIDv4`.
fn is_valid(uuid: &uuid::Uuid) -> bool {
    uuid != &INVALID_UUID && uuid.get_version_num() == UuidV4::UUID_VERSION_NUMBER
}

impl Display for UuidV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Decode<'_, CborContext> for UuidV4 {
    fn decode(d: &mut Decoder<'_>, ctx: &mut CborContext) -> Result<Self, minicbor::decode::Error> {
        let uuid = decode_cbor_uuid(d, ctx)?;
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(minicbor::decode::Error::message(format!(
                "'{uuid}' is not a valid UUIDv4"
            )))
        }
    }
}

impl Encode<CborContext> for UuidV4 {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut CborContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_cbor_uuid(self.uuid(), e, ctx)
    }
}

/// Returns a `UUIDv4` from `uuid::Uuid`.
impl TryFrom<uuid::Uuid> for UuidV4 {
    type Error = anyhow::Error;

    fn try_from(uuid: uuid::Uuid) -> Result<Self, Self::Error> {
        anyhow::ensure!(is_valid(&uuid), "'{uuid}' is not a valid UUIDv4");
        Ok(Self(uuid))
    }
}

/// Returns a `uuid::Uuid` from `UUIDv4`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<UuidV4> for uuid::Uuid {
    fn from(value: UuidV4) -> Self {
        value.0
    }
}

impl<'de> serde::Deserialize<'de> for UuidV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let uuid = uuid::Uuid::deserialize(deserializer)?;
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(serde::de::Error::custom(format!(
                "'{uuid}' is not a valid UUIDv4"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_invalid_uuid() {
        let invalid_uuid = UuidV4::invalid();
        assert!(!invalid_uuid.is_valid(), "Invalid UUID should not be valid");
        assert_eq!(
            invalid_uuid.uuid(),
            INVALID_UUID,
            "Invalid UUID should match INVALID_UUID"
        );
    }

    #[test]
    fn test_valid_uuid() {
        let valid_uuid = UuidV4::try_from(Uuid::new_v4()).unwrap();
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");

        let valid_uuid = UuidV4::new();
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");
    }

    #[test]
    fn test_invalid_version_uuid() {
        assert!(
            UuidV4::try_from(INVALID_UUID).is_err(),
            "Zero UUID should not be valid"
        );
    }
}
