//! `UUIDv7` Type.
use std::fmt::{Display, Formatter};

use minicbor::{Decode, Decoder, Encode};

use super::{decode_cbor_uuid, encode_cbor_uuid, CborContext, INVALID_UUID};

/// Type representing a `UUIDv7`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Serialize)]
pub struct UuidV7(uuid::Uuid);

impl UuidV7 {
    /// Version for `UUIDv7`.
    const UUID_VERSION_NUMBER: usize = 7;

    /// Generates a random `UUIDv4`.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Generates a zeroed out `UUIDv7` that can never be valid.
    #[must_use]
    pub fn invalid() -> Self {
        Self(INVALID_UUID)
    }

    /// Check if this is a valid `UUIDv7`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        is_valid(&self.0)
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        self.0
    }
}

/// Check if this is a valid `UUIDv7`.
fn is_valid(uuid: &uuid::Uuid) -> bool {
    uuid != &INVALID_UUID && uuid.get_version_num() == UuidV7::UUID_VERSION_NUMBER
}

impl Display for UuidV7 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Decode<'_, CborContext> for UuidV7 {
    fn decode(d: &mut Decoder<'_>, ctx: &mut CborContext) -> Result<Self, minicbor::decode::Error> {
        let uuid = decode_cbor_uuid(d, ctx)?;
        Ok(Self(uuid))
    }
}

impl Encode<CborContext> for UuidV7 {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut CborContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_cbor_uuid(self.uuid(), e, ctx)
    }
}

/// Returns a `UUIDv7` from `uuid::Uuid`.
impl TryFrom<uuid::Uuid> for UuidV7 {
    type Error = anyhow::Error;

    fn try_from(uuid: uuid::Uuid) -> Result<Self, Self::Error> {
        anyhow::ensure!(is_valid(&uuid), "'{uuid}' is not a valid UUIDv7");
        Ok(Self(uuid))
    }
}

/// Returns a `uuid::Uuid` from `UUIDv7`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<UuidV7> for uuid::Uuid {
    fn from(value: UuidV7) -> Self {
        value.0
    }
}

impl<'de> serde::Deserialize<'de> for UuidV7 {
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
        let invalid_uuid = UuidV7::invalid();
        assert!(!invalid_uuid.is_valid(), "Invalid UUID should not be valid");
        assert_eq!(
            invalid_uuid.uuid(),
            INVALID_UUID,
            "Invalid UUID should match INVALID_UUID"
        );
    }

    #[test]
    fn test_valid_uuid() {
        let valid_uuid = UuidV7::try_from(Uuid::now_v7()).unwrap();
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");

        let valid_uuid = UuidV7::new();
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");
    }

    #[test]
    fn test_invalid_version_uuid() {
        assert!(
            UuidV7::try_from(INVALID_UUID).is_err(),
            "Zero UUID should not be valid"
        );
    }
}
