//! `UUIDv7` Type.
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use minicbor::{Decode, Decoder, Encode};
use uuid::Uuid;

use super::{CborContext, decode_cbor_uuid, encode_cbor_uuid};

/// Type representing a `UUIDv7`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, serde::Serialize)]
pub struct UuidV7(Uuid);

/// `UUIDv7` invalid error
#[derive(Debug, Clone, thiserror::Error)]
#[error("'{0}' is not a valid UUIDv7")]
pub struct InvalidUuidV7(uuid::Uuid);

impl UuidV7 {
    /// Version for `UUIDv7`.
    const UUID_VERSION_NUMBER: usize = 7;

    /// Generates a random `UUIDv4`.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// A const alternative impl of `TryFrom<Uuid>`
    ///
    /// # Errors
    ///   - `InvalidUuidV7`
    pub const fn try_from_uuid(uuid: Uuid) -> Result<Self, InvalidUuidV7> {
        if is_valid(&uuid) {
            Ok(Self(uuid))
        } else {
            Err(InvalidUuidV7(uuid))
        }
    }
}

/// Check if this is a valid `UUIDv7`.
const fn is_valid(uuid: &Uuid) -> bool {
    uuid.get_version_num() == UuidV7::UUID_VERSION_NUMBER
}

impl Display for UuidV7 {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Decode<'_, CborContext> for UuidV7 {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut CborContext,
    ) -> Result<Self, minicbor::decode::Error> {
        let uuid = decode_cbor_uuid(d, ctx)?;
        Self::try_from_uuid(uuid).map_err(minicbor::decode::Error::message)
    }
}

impl Encode<CborContext> for UuidV7 {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut CborContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_cbor_uuid(self.uuid(), e, ctx)
    }
}

/// Returns a `UUIDv7` from `uuid::Uuid`.
impl TryFrom<Uuid> for UuidV7 {
    type Error = InvalidUuidV7;

    fn try_from(uuid: Uuid) -> Result<Self, Self::Error> {
        Self::try_from_uuid(uuid)
    }
}

/// Returns a `uuid::Uuid` from `UUIDv7`.
///
/// NOTE: This does not guarantee that the `UUID` is valid.
impl From<UuidV7> for Uuid {
    fn from(value: UuidV7) -> Self {
        value.0
    }
}

impl<'de> serde::Deserialize<'de> for UuidV7 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let uuid = Uuid::deserialize(deserializer)?;
        Self::try_from_uuid(uuid).map_err(serde::de::Error::custom)
    }
}

/// `FromStr` invalid error
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParsingError {
    /// `UUIDv7` invalid error
    #[error(transparent)]
    InvalidUuidV7(#[from] InvalidUuidV7),
    /// Invalid string conversion
    #[error("Invalid string conversion: {0}")]
    StringConversion(String),
}

impl FromStr for UuidV7 {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s).map_err(|_| ParsingError::StringConversion(s.to_string()))?;
        Ok(Self::try_from_uuid(uuid)?)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::uuid::INVALID_UUID;

    #[test]
    fn test_invalid_uuid() {
        assert!(UuidV7::try_from(Uuid::new_v4()).is_err());

        assert!(
            UuidV7::try_from(INVALID_UUID).is_err(),
            "Zero UUID should not be valid"
        );
    }

    #[test]
    fn test_valid_uuid() {
        assert!(UuidV7::try_from(Uuid::now_v7()).is_ok());
    }
}
