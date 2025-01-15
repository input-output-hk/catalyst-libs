//! `UUID` types.

use displaydoc::Display;
use minicbor::{data::Tag, Decoder};
use thiserror::Error;
use uuid::Uuid;

mod uuid_v4;
mod uuid_v7;

pub use uuid_v4::UuidV4 as V4;
pub use uuid_v7::UuidV7 as V7;

/// Invalid Doc Type UUID
pub const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

/// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
pub(crate) const UUID_CBOR_TAG: u64 = 37;

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
    let tag = d.tag()?;
    if UUID_CBOR_TAG != tag.as_u64() {
        return Err(minicbor::decode::Error::message(format!(
            "tag value must be: {UUID_CBOR_TAG}, provided: {}",
            tag.as_u64(),
        )));
    }
    let decoded = d
        .bytes()?
        .try_into()
        .map_err(|_| minicbor::decode::Error::message("Expected UUID to have 16 bytes"))?;
    let uuid = uuid::Uuid::from_bytes(decoded);
    Ok(uuid)
}

/// Encode `UUID` into `CBOR`
pub(crate) fn encode_cbor_uuid<W: minicbor::encode::Write>(
    uuid: uuid::Uuid, e: &mut minicbor::Encoder<W>, (): &mut (),
) -> Result<(), minicbor::encode::Error<W::Error>> {
    e.tag(Tag::new(UUID_CBOR_TAG))?;
    e.bytes(uuid.as_bytes())?;
    Ok(())
}
