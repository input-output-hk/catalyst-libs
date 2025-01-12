//! `UUID` types.

use displaydoc::Display;
use thiserror::Error;
use uuid::Uuid;

mod uuid_v4;
mod uuid_v7;

pub use uuid_v4::UuidV4 as V4;
pub use uuid_v7::UuidV7 as V7;

/// Invalid Doc Type UUID
pub const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

/// CBOR tag for UUID content.
pub const UUID_CBOR_TAG: u64 = 37;

/// Errors that can occur when decoding CBOR-encoded UUIDs.
#[derive(Display, Debug, Error)]
pub enum CborUuidError {
    /// Invalid CBOR encoded UUID type
    InvalidCborType,
    /// Invalid CBOR encoded UUID type: invalid bytes size
    InvalidByteSize,
    /// UUID {uuid} is not `v{expected_version}`
    InvalidVersion {
        /// The decoded UUID that was checked.
        uuid: Uuid,
        /// The expected version of the UUID, which did not match the decoded one.
        expected_version: usize,
    },
}

/// Encode `UUID` into `CBOR`.
pub(crate) fn encode_cbor_uuid(uuid: uuid::Uuid) -> coset::cbor::Value {
    coset::cbor::Value::Tag(
        UUID_CBOR_TAG,
        coset::cbor::Value::Bytes(uuid.as_bytes().to_vec()).into(),
    )
}

/// Decode `CBOR` encoded `UUID`.
pub(crate) fn decode_cbor_uuid(val: &coset::cbor::Value) -> Result<uuid::Uuid, CborUuidError> {
    let Some((UUID_CBOR_TAG, coset::cbor::Value::Bytes(bytes))) = val.as_tag() else {
        return Err(CborUuidError::InvalidCborType);
    };
    let uuid = uuid::Uuid::from_bytes(
        bytes
            .clone()
            .try_into()
            .map_err(|_| CborUuidError::InvalidByteSize)?,
    );
    Ok(uuid)
}
