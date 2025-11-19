//! CBOR decoding helper functions.

use minicbor::{Decoder, data::Tag, decode};

/// Generic helper function for decoding different types.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_helper<'a, T, C>(
    d: &mut Decoder<'a>,
    from: &str,
    context: &mut C,
) -> Result<T, decode::Error>
where
    T: minicbor::Decode<'a, C>,
{
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
pub fn decode_bytes(
    d: &mut Decoder,
    from: &str,
) -> Result<Vec<u8>, decode::Error> {
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
pub fn decode_array_len(
    d: &mut Decoder,
    from: &str,
) -> Result<u64, decode::Error> {
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
pub fn decode_map_len(
    d: &mut Decoder,
    from: &str,
) -> Result<u64, decode::Error> {
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
pub fn decode_tag(
    d: &mut Decoder,
    from: &str,
) -> Result<Tag, decode::Error> {
    d.tag()
        .map_err(|e| decode::Error::message(format!("Failed to decode tag in {from}: {e}")))
}

/// Decode any in CDDL (any CBOR type) and return its bytes.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_any<'d>(
    d: &mut Decoder<'d>,
    from: &str,
) -> Result<&'d [u8], decode::Error> {
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

/// Extracts the raw bytes of a CBOR map from a decoder based on specified positions.
/// This function retrieves the raw byte representation of a CBOR map between the given
/// start and end positions from the decoder's underlying buffer.
///
/// # Errors
///  - Invalid map byte range: indices out of bounds
pub fn get_bytes<'a>(
    d: &Decoder<'a>,
    map_start: usize,
    map_end: usize,
) -> Result<&'a [u8], decode::Error> {
    d.input()
        .get(map_start..map_end)
        .ok_or_else(|| decode::Error::message("Invalid map byte range: indices out of bounds"))
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
