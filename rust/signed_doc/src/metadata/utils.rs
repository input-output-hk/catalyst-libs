//! Utitlity functions for metadata decoding fields

use catalyst_types::uuid::CborContext;
use coset::CborSerializable;

/// Find a value for a predicate in the protected header.
pub(crate) fn cose_protected_header_find(
    protected: &coset::ProtectedHeader, mut predicate: impl FnMut(&coset::Label) -> bool,
) -> Option<&coset::cbor::Value> {
    protected
        .header
        .rest
        .iter()
        .find(|(key, _)| predicate(key))
        .map(|(_, value)| value)
}

/// Encode `uuid::Uuid` type into `coset::cbor::Value`.
///
/// This is used to encode `UuidV4` and `UuidV7` types.
pub(crate) fn encode_cbor_uuid<T: minicbor::encode::Encode<CborContext>>(
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
pub(crate) fn decode_cbor_uuid<T: for<'a> minicbor::decode::Decode<'a, CborContext>>(
    value: coset::cbor::Value,
) -> anyhow::Result<T> {
    match value.to_vec() {
        Ok(cbor_value) => {
            minicbor::decode_with(&cbor_value, &mut CborContext::Tagged)
                .map_err(|e| anyhow::anyhow!("Invalid UUID, err: {e}"))
        },
        Err(e) => anyhow::bail!("Invalid CBOR value, err: {e}"),
    }
}
