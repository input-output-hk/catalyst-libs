//! `UUIDv4` Type.
use std::fmt::{Display, Formatter};

use minicbor::{Decode, Decoder, Encode};
use scylla::{
    _macro_internal::{
        CellWriter, ColumnType, DeserializationError, FrameSlice, SerializationError,
        SerializeValue, TypeCheckError, WrittenCellProof,
    },
    deserialize::DeserializeValue,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{decode_cbor_uuid, encode_cbor_uuid, CborContext, UuidError, INVALID_UUID};

/// Type representing a `UUIDv4`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Serialize)]
pub struct UuidV4(Uuid);

impl UuidV4 {
    /// Version for `UUIDv4`.
    const UUID_VERSION_NUMBER: usize = 4;

    /// Generates a random `UUIDv4`.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
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
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

/// Check if this is a valid `UUIDv4`.
fn is_valid(uuid: &Uuid) -> bool {
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
            Err(minicbor::decode::Error::message(UuidError::InvalidUuidV4(
                uuid,
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
impl TryFrom<Uuid> for UuidV4 {
    type Error = UuidError;

    fn try_from(uuid: Uuid) -> Result<Self, Self::Error> {
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(UuidError::InvalidUuidV4(uuid))
        }
    }
}

/// Returns a `uuid::Uuid` from `UUIDv4`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<UuidV4> for Uuid {
    fn from(value: UuidV4) -> Self {
        value.0
    }
}

impl<'de> serde::Deserialize<'de> for UuidV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let uuid = <Uuid as Deserialize>::deserialize(deserializer)?;
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(serde::de::Error::custom(UuidError::InvalidUuidV4(uuid)))
        }
    }
}

impl SerializeValue for UuidV4 {
    fn serialize<'b>(
        &self, typ: &ColumnType, writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for UuidV4 {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        Uuid::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>, v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let uuid = <Uuid as DeserializeValue>::deserialize(typ, v)?;
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(DeserializationError::new(UuidError::InvalidUuidV4(uuid)))
        }
    }
}

#[cfg(test)]
mod tests {
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
