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

// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
// pub(crate) const UUID_CBOR_TAG: u64 = 37;

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
pub(crate) fn decode_cbor_uuid<C>(
    d: &mut Decoder<'_>, ctx: &mut C,
) -> Result<uuid::Uuid, minicbor::decode::Error> {
    let decoded_vec: Vec<u8> = d.decode_with(ctx)?;
    let decoded = decoded_vec.try_into().map_err(|e| {
        minicbor::decode::Error::message(format!(
            "Expected UUID to have 16 bytes, err: {}",
            hex::encode(e)
        ))
    })?;
    let uuid = uuid::Uuid::from_bytes(decoded);
    Ok(uuid)
}

/// Encode `UUID` into `CBOR`
pub(crate) fn encode_cbor_uuid<C, W: minicbor::encode::Write>(
    uuid: uuid::Uuid, e: &mut minicbor::Encoder<W>, ctx: &mut C,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    e.encode_with(uuid.as_bytes(), ctx)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use minicbor::data::Tagged;

    use super::{V4, V7};

    const UUID_CBOR_TAG: u64 = 37;

    #[test]
    fn test_cbor_uuid_v4_roundtrip() {
        let uuid: V4 = uuid::Uuid::new_v4().into();
        let mut bytes = Vec::new();
        minicbor::encode(uuid, &mut bytes).unwrap();
        let decoded = minicbor::decode(bytes.as_slice()).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_cbor_uuid_v7_roundtrip() {
        let uuid: V7 = uuid::Uuid::now_v7().into();
        let mut bytes = Vec::new();
        minicbor::encode(uuid, &mut bytes).unwrap();
        let decoded = minicbor::decode(bytes.as_slice()).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_cbor_tagged_uuid_v4_roundtrip() {
        let uuid: V4 = uuid::Uuid::new_v4().into();
        let tagged: Tagged<UUID_CBOR_TAG, V4> = uuid.into();
        let mut bytes = Vec::new();
        minicbor::encode(tagged, &mut bytes).unwrap();
        let decoded = minicbor::decode(bytes.as_slice()).unwrap();
        assert_eq!(tagged, decoded);
    }

    #[test]
    fn test_cbor_tagged_uuid_v7_roundtrip() {
        let uuid: V7 = uuid::Uuid::now_v7().into();
        let tagged: Tagged<UUID_CBOR_TAG, V7> = uuid.into();
        let mut bytes = Vec::new();
        minicbor::encode(tagged, &mut bytes).unwrap();
        let decoded = minicbor::decode(bytes.as_slice()).unwrap();
        assert_eq!(tagged, decoded);
    }
}
