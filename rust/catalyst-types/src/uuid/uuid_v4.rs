//! `UUIDv4` Type.
use std::fmt::{Display, Formatter};

use minicbor::{Decode, Decoder, Encode};

use super::{decode_cbor_uuid, encode_cbor_uuid, INVALID_UUID};

/// Type representing a `UUIDv4`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[serde(from = "uuid::Uuid")]
#[serde(into = "uuid::Uuid")]
pub struct UuidV4 {
    /// UUID
    uuid: uuid::Uuid,
}

impl UuidV4 {
    /// Version for `UUIDv4`.
    const UUID_VERSION_NUMBER: usize = 4;

    /// Generates a zeroed out `UUIDv4` that can never be valid.
    #[must_use]
    pub fn invalid() -> Self {
        Self { uuid: INVALID_UUID }
    }

    /// Check if this is a valid `UUIDv4`.
    #[must_use]
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

impl<C> Decode<'_, C> for UuidV4 {
    fn decode(d: &mut Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let uuid = decode_cbor_uuid(d, ctx)?;
        Ok(Self { uuid })
    }
}

impl<C> Encode<C> for UuidV4 {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_cbor_uuid(self.uuid(), e, ctx)
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

/// Returns a `uuid::Uuid` from `UUIDv4`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<UuidV4> for uuid::Uuid {
    fn from(value: UuidV4) -> Self {
        value.uuid
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
        let valid_uuid = UuidV4::from(Uuid::new_v4());
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");
    }

    #[test]
    fn test_invalid_version_uuid() {
        let invalid_version_uuid = UuidV4::from(Uuid::from_u128(0));
        assert!(
            !invalid_version_uuid.is_valid(),
            "Zero UUID should not be valid"
        );
    }
}
