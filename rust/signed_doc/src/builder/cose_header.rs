use super::{cbor_map::CborMap, EncodeError};

/// The KID label as per [RFC 8152 3.1 section](https://datatracker.ietf.org/doc/html/rfc8152#section-3.1).
pub const KID_LABEL: u8 = 4;

/// Since [`KID_LABEL`] fits into `0 ..= 0x17`, it is encoded as is.
const fn kid_label_bytes() -> &'static [u8] {
    &[KID_LABEL]
}

/// Make the header using the provided cbor-encoded key-value pairs representing
/// fields, conforming to the header fields specification.
pub fn make_metadata_header(metadata_fields: &CborMap) -> Vec<u8> {
    let mut encoder = minicbor::Encoder::new(vec![]);

    let map_len = u64::try_from(metadata_fields.len()).unwrap_or(u64::MAX);
    encoder.map(map_len);
    for (encoded_key, encoded_v) in metadata_fields.iter() {
        // Writing a pre-encoded field of the map.
        encoder.writer_mut().extend_from_slice(encoded_key);
        encoder.writer_mut().extend_from_slice(encoded_v);
    }

    encoder.into_writer()
}

/// Make the protected header for the `Cose_signature`, conforming to the header fields specification.
///
/// # Errors
///
/// - If encoding of the `kid` fails.
pub fn make_signature_header(kid: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = minicbor::Encoder::new(vec![]);
    // A map with a single `kid` field.
    encoder.map(1u64)?.u8(KID_LABEL)?.bytes(kid)?;
    Ok(encoder.into_writer())
}
