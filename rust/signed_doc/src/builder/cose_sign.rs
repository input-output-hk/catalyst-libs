use minicbor::bytes::{ByteArray, ByteSlice};

use super::EncodeError;

/// Make `Cose_signature`.
///
/// Signature bytes should represent a cryptographic signature.
pub fn _make_cose_signature(
    protected_header: &ByteSlice, signature_bytes: &ByteSlice,
) -> Result<Vec<u8>, EncodeError> {
    minicbor::to_vec([protected_header, signature_bytes])
}

/// Collect an array from an iterator of pre-encoded `Cose_signature` items.
///
/// Signature bytes should represent a cryptographic signature.
pub fn _collect_cose_signature_array<S>(signatures: S) -> Result<Vec<u8>, EncodeError>
where
    S: IntoIterator<Item: AsRef<ByteSlice>, IntoIter: ExactSizeIterator>,
{
    let iter = signatures.into_iter();
    let array_len = u64::try_from(iter.len().saturating_add(1)).unwrap_or(u64::MAX);
    let mut encoder = minicbor::Encoder::new(vec![]);
    encoder.array(array_len)?;
    for signature in iter {
        encoder.bytes(signature.as_ref())?;
    }
    Ok(encoder.into_writer())
}

/// Make cbor-encoded `Cose_Sign`.
pub fn _make_cose_sign<S>(
    protected_header: &ByteSlice, content: &ByteSlice, signatures: S,
) -> Result<Vec<u8>, EncodeError>
where
    S: IntoIterator<Item: AsRef<ByteSlice>, IntoIter: ExactSizeIterator>,
{
    minicbor::to_vec((
        protected_header,
        ByteArray::from([]), // unprotected.
        content,
        _collect_cose_signature_array(signatures)?,
    ))
}

/// Create a binary blob that will be signed and construct the to-be-signed data from it
/// in-place.
pub fn make_tbs_data(
    metadata_header: &[u8], signature_header: &[u8], content: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    /// The context string as per [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4).
    const SIGNATURE_CONTEXT: &str = "Signature";

    minicbor::to_vec((
        SIGNATURE_CONTEXT,
        <&ByteSlice>::from(metadata_header),
        <&ByteSlice>::from(signature_header),
        ByteArray::from([]), // aad.
        <&ByteSlice>::from(content),
    ))
}
