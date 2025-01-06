//! `UUIDv7` Type.
use std::fmt::{Display, Formatter};

use super::{decode_cbor_uuid, INVALID_UUID};

/// Type representing a `UUIDv7`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(from = "uuid::Uuid")]
pub struct UuidV7 {
    /// UUID
    uuid: uuid::Uuid,
}

impl UuidV7 {
    /// Version for `UUIDv7`.
    const UUID_VERSION_NUMBER: usize = 7;

    /// Generates a zeroed out `UUIDv7` that can never be valid.
    #[must_use]
    pub fn invalid() -> Self {
        Self { uuid: INVALID_UUID }
    }

    /// Check if this is a valid `UUIDv7`.
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

impl Display for UuidV7 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.uuid)
    }
}

impl TryFrom<&coset::cbor::Value> for UuidV7 {
    type Error = super::CborUuidError;

    fn try_from(cbor_value: &coset::cbor::Value) -> Result<Self, Self::Error> {
        match decode_cbor_uuid(cbor_value) {
            Ok(uuid) => {
                if uuid.get_version_num() == Self::UUID_VERSION_NUMBER {
                    Ok(Self { uuid })
                } else {
                    Err(super::CborUuidError::InvalidVersion {
                        uuid,
                        expected_version: Self::UUID_VERSION_NUMBER,
                    })
                }
            },
            Err(e) => Err(e),
        }
    }
}

/// Returns a `UUIDv7` from `uuid::Uuid`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<uuid::Uuid> for UuidV7 {
    fn from(uuid: uuid::Uuid) -> Self {
        Self { uuid }
    }
}

#[cfg(test)]
mod tests {
    use coset::cbor::Value;
    use uuid::Uuid;

    use super::*;
    use crate::uuid::UUID_CBOR_TAG;

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
        let valid_uuid = UuidV7::from(
            Uuid::try_parse("017f22e3-79b0-7cc7-98cf-e0bbf8a1c5f1").unwrap(),
        );
        assert!(valid_uuid.is_valid(), "Valid UUID should be valid");
    }

    #[test]
    fn test_invalid_version_uuid() {
        let invalid_version_uuid = UuidV7::from(Uuid::from_u128(0));
        assert!(
            !invalid_version_uuid.is_valid(),
            "Zero UUID should not be valid"
        );
    }

    #[test]
    fn test_try_from_cbor_valid_uuid() {
        let uuid = Uuid::try_parse("017f22e3-79b0-7cc7-98cf-e0bbf8a1c5f1").unwrap();
        let cbor_value = Value::Tag(
            UUID_CBOR_TAG,
            Box::new(Value::Bytes(uuid.as_bytes().to_vec())),
        );
        let result = UuidV7::try_from(&cbor_value);

        assert!(
            result.is_ok(),
            "Should successfully parse valid UUID from CBOR"
        );
        let uuid_v7 = result.unwrap();
        assert!(uuid_v7.is_valid(), "Parsed UUIDv7 should be valid");
        assert_eq!(
            uuid_v7.uuid(),
            uuid,
            "Parsed UUID should match original UUID"
        );
    }

    #[test]
    fn test_try_from_cbor_invalid_uuid() {
        let cbor_value = Value::Bytes(vec![0; 16]);
        let result = UuidV7::try_from(&cbor_value);

        assert!(
            result.is_err(),
            "Should fail to parse invalid UUID from CBOR"
        );
    }
}
