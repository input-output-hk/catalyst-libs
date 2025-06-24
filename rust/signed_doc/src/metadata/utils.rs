//! Utility functions for metadata decoding fields

use catalyst_types::uuid::{CborContext, UuidV7};
use coset::CborSerializable;

/// A convenient wrapper over the `UuidV7` type, to implement
/// `TryFrom<coset::cbor::Value>` and `TryFrom<Self> for coset::cbor::Value` traits.
pub(crate) struct CborUuidV7(pub(crate) UuidV7);
impl TryFrom<&coset::cbor::Value> for CborUuidV7 {
    type Error = anyhow::Error;

    fn try_from(value: &coset::cbor::Value) -> Result<Self, Self::Error> {
        Ok(Self(decode_cbor_uuid(value)?))
    }
}
impl TryFrom<CborUuidV7> for coset::cbor::Value {
    type Error = anyhow::Error;

    fn try_from(value: CborUuidV7) -> Result<Self, Self::Error> {
        encode_cbor_uuid(value.0)
    }
}

/// Encode `uuid::Uuid` type into `coset::cbor::Value`.
///
/// This is used to encode `UuidV4` and `UuidV7` types.
fn encode_cbor_uuid<T: minicbor::encode::Encode<CborContext>>(
    value: T,
) -> anyhow::Result<coset::cbor::Value> {
    let mut cbor_bytes = Vec::new();
    minicbor::encode_with(value, &mut cbor_bytes, &mut CborContext::Tagged)
        .map_err(|e| anyhow::anyhow!("Unable to encode CBOR value, err: {e}"))?;
    coset::cbor::Value::from_slice(&cbor_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid CBOR value, err: {e}"))
}

/// Decode `From<uuid::Uuid>` type from `coset::cbor::Value`.
///
/// This is used to decode `UuidV4` and `UuidV7` types.
fn decode_cbor_uuid<T: for<'a> minicbor::decode::Decode<'a, CborContext>>(
    value: &coset::cbor::Value,
) -> anyhow::Result<T> {
    let mut cbor_bytes = Vec::new();
    coset::cbor::ser::into_writer(value, &mut cbor_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid CBOR value, err: {e}"))?;
    minicbor::decode_with(&cbor_bytes, &mut CborContext::Tagged)
        .map_err(|e| anyhow::anyhow!("Invalid UUID, err: {e}"))
}
