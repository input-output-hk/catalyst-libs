//! `UUIDv7` Type.
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use minicbor::{Decode, Decoder, Encode};
use uuid::Uuid;

use super::{CborContext, decode_cbor_uuid, encode_cbor_uuid};

/// Type representing a `UUIDv7`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct UuidV7(Uuid, DateTime<Utc>);

/// `UUIDv7` invalid error
#[derive(Debug, Clone, thiserror::Error)]
#[error("'{0}' is not a valid UUIDv7")]
pub struct InvalidUuidV7(uuid::Uuid);

impl UuidV7 {
    /// Version for `UUIDv7`.
    const UUID_VERSION_NUMBER: usize = 7;

    /// Generates a random `UUIDv7`.
    ///
    /// # Panics
    ///  `Utc::now()` returns system before Unix epoch, cannot happen.
    #[must_use]
    #[allow(clippy::new_without_default, clippy::expect_used)]
    pub fn new() -> Self {
        let uuid = Uuid::now_v7();
        let dt =
            uuid_v7_to_datetime(&uuid).expect("cannot retrieve datetime from the valid UUID V7");
        Self(uuid, dt)
    }

    /// Returns the corresponding `DateTime<Utc>`.
    #[must_use]
    pub fn time(&self) -> &DateTime<Utc> {
        &self.1
    }

    /// Returns the `uuid::Uuid` type.
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.0
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
        Self::try_from(uuid).map_err(minicbor::decode::Error::message)
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
        if is_valid(&uuid) {
            let datetime = uuid_v7_to_datetime(&uuid)?;
            Ok(Self(uuid, datetime))
        } else {
            Err(InvalidUuidV7(uuid))
        }
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
        Self::try_from(uuid).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for UuidV7 {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
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
        Ok(Self::try_from(uuid)?)
    }
}

/// Retrieving `DateTime<Utc>` from a uuid timestamp component.
fn uuid_v7_to_datetime(v: &Uuid) -> Result<DateTime<Utc>, InvalidUuidV7> {
    let (time_secs, time_nanos) = v.get_timestamp().ok_or(InvalidUuidV7(*v))?.to_unix();
    i64::try_from(time_secs)
        .ok()
        .and_then(|time_secs| DateTime::from_timestamp(time_secs, time_nanos))
        .ok_or(InvalidUuidV7(*v))
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

    #[test]
    fn datetime_and_uuid_timestamp_alignment() {
        let uuid = UuidV7::new();
        let uuid_timestamp = uuid.uuid().get_timestamp().unwrap();

        assert_eq!(uuid.to_string().parse::<UuidV7>().unwrap(), uuid);
        assert_eq!(
            u64::try_from(uuid.time().timestamp()).unwrap(),
            uuid_timestamp.to_unix().0
        );
        assert_eq!(
            uuid.time().timestamp_subsec_nanos(),
            uuid_timestamp.to_unix().1
        );
    }
}
