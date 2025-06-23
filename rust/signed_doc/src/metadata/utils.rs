//! Utility functions for metadata decoding fields

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, UuidV7},
};
use coset::{CborSerializable, Label, ProtectedHeader};
use minicbor::{Decode, Decoder};

/// Decode cose protected header value using minicbor decoder.
pub(crate) fn decode_cose_protected_header_value<C, T>(
    protected: &ProtectedHeader, context: &mut C, label: &str,
) -> Option<T>
where T: for<'a> Decode<'a, C> {
    cose_protected_header_find(protected, |key| matches!(key, Label::Text(l) if l == label))
        .and_then(|value| {
            let bytes = value.clone().to_vec().unwrap_or_default();
            Decoder::new(&bytes).decode_with(context).ok()
        })
}

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

/// Tries to decode field by the `field_name` from the COSE protected header
pub(crate) fn decode_document_field_from_protected_header<T>(
    protected: &ProtectedHeader, field_name: &str, report_content: &str, report: &ProblemReport,
) -> Option<T>
where T: for<'a> TryFrom<&'a coset::cbor::Value> {
    if let Some(cbor_doc_field) =
        cose_protected_header_find(protected, |key| key == &Label::Text(field_name.to_string()))
    {
        if let Ok(field) = T::try_from(cbor_doc_field) {
            return Some(field);
        }
        report.conversion_error(
            &format!("CBOR COSE protected header {field_name}"),
            &format!("{cbor_doc_field:?}"),
            "Expected a CBOR UUID",
            &format!("{report_content}, decoding CBOR UUID for {field_name}",),
        );
    }
    None
}

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
