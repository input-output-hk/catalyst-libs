//! `UUID` types.

mod uuid_v4;
mod uuid_v7;

pub use uuid_v4::UuidV4;
pub use uuid_v7::UuidV7;

/// Invalid Doc Type UUID
pub const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

/// CBOR tag for UUID content.
pub const UUID_CBOR_TAG: u64 = 37;

/// Decode `CBOR` encoded `UUID`.
pub(crate) fn decode_cbor_uuid(val: &coset::cbor::Value) -> anyhow::Result<uuid::Uuid> {
    let Some((UUID_CBOR_TAG, coset::cbor::Value::Bytes(bytes))) = val.as_tag() else {
        anyhow::bail!("Invalid CBOR encoded UUID type");
    };
    let uuid = uuid::Uuid::from_bytes(
        bytes
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid CBOR encoded UUID type, invalid bytes size"))?,
    );
    Ok(uuid)
}
