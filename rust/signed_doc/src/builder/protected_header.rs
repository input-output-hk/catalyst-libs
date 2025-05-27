use minicbor::{
    bytes::{ByteSlice, ByteVec},
    Encode as _,
};

use super::{cbor_map::CborMap, EncodeError};

/// The KID label as per [RFC 8152 3.1 section](https://datatracker.ietf.org/doc/html/rfc8152#section-3.1).
pub const KID_LABEL: u8 = 4;

/// Since [`KID_LABEL`] fits into `0 ..= 0x17`, it is encoded as is.
const fn kid_label_bytes() -> &'static [u8] {
    &[KID_LABEL]
}

/// Make to-be-signed data using the provided cbor-encoded key-value pairs representing
/// fields, conforming to the protected fields specification.
///
/// The validity of the encoding of the fields argument is not checked.
///
/// `kid` field is encoded in a map, respecting **bytewise** lexicographic key ordering.
///
/// `kid` is encoded as a byte string as is.
///
/// # Errors
///
/// - If encoding of the `kid` fails.
/// - If `kid` field is found in the metadata fields.
pub fn make_protected_header(
    kid: &ByteSlice, metadata_fields: &CborMap,
) -> Result<ByteVec, EncodeError> {
    let mut encoder = minicbor::Encoder::new(vec![]);
    let mut fields_iter = metadata_fields.iter();
    // Incrementing to include `kid` entry.
    let map_len = u64::try_from(metadata_fields.len().saturating_add(1)).unwrap_or(u64::MAX);
    encoder.map(map_len);
    // Peeking through the metadata fields to insert `kid` in order.
    loop {
        let next_field = fields_iter.next();
        if next_field.is_some_and(|(encoded_key, _)| encoded_key == kid_label_bytes()) {
            return Err(EncodeError::message(
                "kid label found in the metadata fields",
            ));
        }
        // Writing `kid` where it would have been with *bytewise** lexicographic order.
        if next_field.is_none_or(|(encoded_key, _)| encoded_key > kid_label_bytes()) {
            KID_LABEL.encode(&mut encoder, &mut ())?;
            kid.encode(&mut encoder, &mut ())?;
        }
        let Some((encoded_key, encoded_v)) = next_field else {
            break;
        };
        // Writing a pre-encoded field of the map.
        encoder.writer_mut().extend_from_slice(encoded_key);
        encoder.writer_mut().extend_from_slice(encoded_v);
    }

    Ok(ByteVec::from(encoder.into_writer()))
}
