use minicbor::{
    bytes::{ByteArray, ByteSlice},
    data::Tagged,
    Encode as _,
};

use super::VecEncodeError;

/// Encode headers using the provided cbor-encoded key-value pairs,
/// conforming to the [RFC 8152 specification](https://datatracker.ietf.org/doc/html/rfc8152#autoid-8).
pub fn encode_headers<'a, I>(iter: I) -> Vec<u8>
where
    I: IntoIterator<Item = (&'a [u8], &'a [u8]), IntoIter: ExactSizeIterator>,
{
    let mut encoder = minicbor::Encoder::new(vec![]);

    let iter = iter.into_iter();
    let map_len = u64::try_from(iter.len()).unwrap_or(u64::MAX);
    encoder.map(map_len);

    for (encoded_key, encoded_v) in iter {
        // Writing a pre-encoded field of the map.
        encoder.writer_mut().extend_from_slice(encoded_key);
        encoder.writer_mut().extend_from_slice(encoded_v);
    }

    encoder.into_writer()
}

/// Encode a single protected `kid` header for the COSE Signature.
///
/// # Errors
///
/// - If encoding of the `kid` fails.
pub fn encoed_kid_header(kid: &[u8]) -> Result<Vec<u8>, VecEncodeError> {
    /// The KID label as per [RFC 8152 3.1 section](https://datatracker.ietf.org/doc/html/rfc8152#section-3.1).
    pub const KID_LABEL: u8 = 4;

    let mut encoder = minicbor::Encoder::new(vec![]);
    // A map with a single `kid` field.
    encoder.map(1u64)?.u8(KID_LABEL)?.bytes(kid)?;
    Ok(encoder.into_writer())
}

/// Create a binary blob that will be signed. No support for unprotected headers.
///
/// Described in [section 2 of RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#section-2).
pub fn encode_tbs_data(
    protected_headers: &[u8], signature_header: &[u8], content: Option<&[u8]>,
) -> Result<Vec<u8>, VecEncodeError> {
    /// The context string as per [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4).
    const SIGNATURE_CONTEXT: &str = "Signature";

    minicbor::to_vec((
        SIGNATURE_CONTEXT,
        <&ByteSlice>::from(protected_headers),
        <&ByteSlice>::from(signature_header),
        ByteArray::from([]),                        // no aad.
        <&ByteSlice>::from(content.unwrap_or(&[])), // allowing no payload (i.e. no content).
    ))
}

/// Encode COSE signature.
///
/// Signature bytes should represent a cryptographic signature.
pub fn encode_cose_signature(
    protected_header: &[u8], signature_bytes: &[u8],
) -> Result<Vec<u8>, VecEncodeError> {
    minicbor::to_vec([
        <&ByteSlice>::from(protected_header),
        <&ByteSlice>::from(signature_bytes),
    ])
}

/// Encode an array from an iterator of pre-encoded COSE Signature items.
fn encode_cose_signature_array<S>(signatures: S) -> Result<Vec<u8>, VecEncodeError>
where
    S: IntoIterator<Item: AsRef<[u8]>, IntoIter: ExactSizeIterator>,
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

/// Make cbor-encoded tagged [RFC9052-CoseSign](https://datatracker.ietf.org/doc/html/rfc9052).
pub fn encode_cose_sign<W: minicbor::encode::Write, S>(
    e: &mut minicbor::encode::Encoder<W>, protected: &[u8], payload: Option<&[u8]>, signatures: S,
) -> Result<(), minicbor::encode::Error<W::Error>>
where
    S: IntoIterator<Item: AsRef<[u8]>, IntoIter: ExactSizeIterator>,
{
    /// From the table in [section 2 of RFC 8152](https://datatracker.ietf.org/doc/html/rfc8152#section-2).
    const COSE_SIGN_TAG: u64 = 98;

    let tagged_array = Tagged::<COSE_SIGN_TAG, _>::new((
        <&ByteSlice>::from(protected),
        ByteArray::from([]),             // unprotected.
        payload.map(<&ByteSlice>::from), // allowing `NULL`.
        encode_cose_signature_array(signatures).map_err(minicbor::encode::Error::custom)?,
    ));
    tagged_array.encode(e, &mut ())
}
