//! `UUID` types.

use displaydoc::Display;
use minicbor::Decoder;
use thiserror::Error;
use uuid::Uuid;

mod uuid_v4;
mod uuid_v7;

pub use uuid_v4::UuidV4 as V4;
pub use uuid_v7::UuidV7 as V7;

/// Invalid Doc Type UUID
pub const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

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

/// Decode from `CBOR` into `UUID`
pub(crate) fn decode_cbor_uuid(
    d: &mut Decoder<'_>, (): &mut (),
) -> Result<uuid::Uuid, minicbor::decode::Error> {
    let decoded = d.bytes()?.try_into().map_err(|e| {
        minicbor::decode::Error::message(format!("Expected UUID to have 16 bytes: {e}"))
    })?;
    let uuid = uuid::Uuid::from_bytes(decoded);
    Ok(uuid)
}

/// Encode `UUID` into `CBOR`
pub(crate) fn encode_cbor_uuid<W: minicbor::encode::Write>(
    uuid: uuid::Uuid, e: &mut minicbor::Encoder<W>, (): &mut (),
) -> Result<(), minicbor::encode::Error<W::Error>> {
    e.bytes(uuid.as_bytes())?;
    Ok(())
}
