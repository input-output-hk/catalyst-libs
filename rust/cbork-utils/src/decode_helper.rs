//! CBOR decoding helper functions.

use std::cmp::Ordering;

use minicbor::{data::Tag, decode, Decoder};

/// Represents a CBOR map key-value pair.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MapEntry {
    /// Raw bytes of the encoded key
    pub key_bytes: Vec<u8>,
    /// Raw bytes of the encoded value
    pub value: Vec<u8>,
}

impl PartialOrd for MapEntry {
    /// Compare map entries according to RFC 8949 Section 4.2.3 rules:
    /// 1. Compare by length of encoded key
    /// 2. If lengths equal, compare byte wise lexicographically
    ///
    /// Returns Some(ordering) since comparison is always defined for these types
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MapEntry {
    /// Compare map entries according to RFC 8949 Section 4.2.3 rules:
    /// 1. Compare by length of encoded key
    /// 2. If lengths equal, compare byte wise lexicographically
    fn cmp(&self, other: &Self) -> Ordering {
        self.key_bytes
            .len()
            .cmp(&other.key_bytes.len())
            .then_with(|| self.key_bytes.cmp(&other.key_bytes))
    }
}

/// Decodes a definite size CBOR map
///
/// Additionally:
/// - Indefinite-length maps are not allowed (Section 4.2.2)
///
/// # Errors
///  - Indefinite size map
///  - Decoding errors
pub fn decode_map(d: &mut Decoder) -> Result<Vec<MapEntry>, minicbor::decode::Error> {
    // From RFC 8949 Section 4.2.2:
    // "Indefinite-length items must be made definite-length items."
    // The specification explicitly prohibits indefinite-length items in
    // deterministic encoding to ensure consistent representation.
    let map_len = d.map()?.ok_or_else(|| {
        minicbor::decode::Error::message(
            "Indefinite-length items must be made definite-length items",
        )
    })?;

    // Decode entries to validate them
    let entries = decode_map_entries(d, map_len, |_| Ok(()))?;

    Ok(entries)
}

/// Decodes all key-value pairs in the map
pub(crate) fn decode_map_entries(
    d: &mut Decoder, length: u64, key_check: impl Fn(&[u8]) -> Result<(), minicbor::decode::Error>,
) -> Result<Vec<MapEntry>, minicbor::decode::Error> {
    let capacity = usize::try_from(length).map_err(|_| {
        minicbor::decode::Error::message("Map length too large for current platform")
    })?;
    let mut entries = Vec::with_capacity(capacity);

    // Decode each key-value pair
    for _ in 0..length {
        // Record the starting position of the key
        let key_start = d.position();

        // Skip over the key to find its end position
        d.skip()?;
        let key_end = d.position();

        // Record the starting position of the value
        let value_start = d.position();

        // Skip over the value to find its end position
        d.skip()?;
        let value_end = d.position();

        // The keys themselves must be deterministically encoded (4.2.1)
        let key_bytes = get_bytes(d, key_start, key_end)?;
        key_check(&key_bytes)?;
        let value = get_bytes(d, value_start, value_end)?;

        entries.push(MapEntry { key_bytes, value });
    }

    Ok(entries)
}

/// Extracts the raw bytes of a CBOR map from a decoder based on specified positions.
/// This function retrieves the raw byte representation of a CBOR map between the given
/// start and end positions from the decoder's underlying buffer.
fn get_bytes(
    d: &Decoder<'_>, map_start: usize, map_end: usize,
) -> Result<Vec<u8>, minicbor::decode::Error> {
    d.input()
        .get(map_start..map_end)
        .ok_or_else(|| {
            minicbor::decode::Error::message("Invalid map byte range: indices out of bounds")
        })
        .map(<[u8]>::to_vec)
}

/// Generic helper function for decoding different types.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_helper<'a, T, C>(
    d: &mut Decoder<'a>, from: &str, context: &mut C,
) -> Result<T, decode::Error>
where T: minicbor::Decode<'a, C> {
    T::decode(d, context).map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode {:?} in {from}: {e}",
            std::any::type_name::<T>()
        ))
    })
}

/// Helper function for decoding bytes.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_bytes(d: &mut Decoder, from: &str) -> Result<Vec<u8>, decode::Error> {
    d.bytes().map(<[u8]>::to_vec).map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode bytes in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding array.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_array_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
    d.array()
        .map_err(|e| {
            decode::Error::message(format!(
                "Failed to decode array in {from}:
            {e}"
            ))
        })?
        .ok_or(decode::Error::message(format!(
            "Failed to decode array in {from}, unexpected indefinite length",
        )))
}

/// Helper function for decoding map.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_map_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
    d.map()
        .map_err(|e| decode::Error::message(format!("Failed to decode map in {from}: {e}")))?
        .ok_or(decode::Error::message(format!(
            "Failed to decode map in {from}, unexpected indefinite length",
        )))
}

/// Helper function for decoding tag.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_tag(d: &mut Decoder, from: &str) -> Result<Tag, decode::Error> {
    d.tag()
        .map_err(|e| decode::Error::message(format!("Failed to decode tag in {from}: {e}")))
}

/// Decode any in CDDL (any CBOR type) and return its bytes.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_any<'d>(d: &mut Decoder<'d>, from: &str) -> Result<&'d [u8], decode::Error> {
    let start = d.position();
    d.skip()?;
    let end = d.position();
    let bytes = d
        .input()
        .get(start..end)
        .ok_or(decode::Error::message(format!(
            "Failed to get any CBOR bytes in {from}. Invalid CBOR bytes."
        )))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use minicbor::Encoder;
    use proptest::property_test;

    use super::*;

    #[property_test]
    fn test_decode_any_bytes(random_bytes: Vec<u8>) {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.bytes(&random_bytes).expect("Error encoding bytes");

        let mut d = Decoder::new(&buf);
        let cbor_bytes = decode_any(&mut d, "test").expect("Error decoding bytes");

        let result = decode_bytes(&mut Decoder::new(cbor_bytes), "test").unwrap();
        assert_eq!(result, random_bytes);
    }

    #[property_test]
    fn test_decode_any_string(random_string: String) {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.str(&random_string).expect("Error encoding string");

        let mut d = Decoder::new(&buf);
        let cbor_bytes = decode_any(&mut d, "test").expect("Error decoding string");

        let result =
            decode_helper::<String, _>(&mut Decoder::new(cbor_bytes), "test", &mut ()).unwrap();
        assert_eq!(result, random_string);
    }

    #[property_test]
    fn test_decode_any_array(random_array: Vec<u8>) {
        // The array should contain a supported type
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.array(random_array.len() as u64)
            .expect("Error encoding array");
        for el in &random_array {
            e.u8(*el).expect("Error encoding u8");
        }
        let mut d = Decoder::new(&buf);
        let cbor_bytes = decode_any(&mut d, "test").expect("Error decoding array");
        // The decode of array is just a length of the array
        let result = decode_array_len(&mut Decoder::new(cbor_bytes), "test").unwrap();
        assert_eq!(result, random_array.len() as u64);
    }

    #[property_test]
    fn test_decode_any_u32(random_u32: u32) {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.u32(random_u32).expect("Error encoding u32");

        let mut d = Decoder::new(&buf);
        let cbor_bytes = decode_any(&mut d, "test").expect("Error decoding u32");

        let result =
            decode_helper::<u32, _>(&mut Decoder::new(cbor_bytes), "test", &mut ()).unwrap();
        assert_eq!(result, random_u32);
    }

    #[property_test]
    fn test_decode_any_i32(random_i32: i32) {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.i32(random_i32).expect("Error encoding i32");
        let mut d = Decoder::new(&buf);
        let cbor_bytes = decode_any(&mut d, "test").expect("Error decoding i32");

        let result =
            decode_helper::<i32, _>(&mut Decoder::new(cbor_bytes), "test", &mut ()).unwrap();
        assert_eq!(result, random_i32);
    }

    #[test]
    fn test_decode_any_not_cbor() {
        let mut d = Decoder::new(&[]);
        let result = decode_any(&mut d, "test");
        // Should print out the error message with the location of the error
        assert!(result.is_err());
    }
}
