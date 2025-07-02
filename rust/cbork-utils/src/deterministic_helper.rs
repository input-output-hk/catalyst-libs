//! CBOR decoding helper functions with deterministic encoding validation.
//!
//! Based on RFC 8949 Section 4.2 "Deterministically Encoded CBOR"
//! Rules for deterministic encoding:
//! 1. Integers must use the smallest possible encoding
//! 2. Lengths of arrays, maps, strings must use the smallest possible encoding
//! 3. Indefinite-length items are not allowed
//! 4. Keys in every map must be sorted in lexicographic order
//! 5. Duplicate keys in maps are not allowed
//! 6. Floating point values must use smallest possible encoding
//! 7. Non-finite floating point values are not allowed (NaN, infinite)

/// Maximum value that can be encoded in a 5-bit additional info field
/// RFC 8949 Section 4.2.1: "0 to 23 must be expressed in the same byte as the major type"
/// Values 0-23 are encoded directly in the additional info field of the initial byte
pub(crate) const CBOR_MAX_TINY_VALUE: u64 = 23;

/// Extracts the declared length from a CBOR data item according to RFC 8949 encoding
/// rules.
///
/// This function analyzes the major type and additional information in the CBOR initial
/// byte to determine if the data item has a declared length and what that length is.
///
/// ## CBOR Major Types and Length Semantics (RFC 8949 Section 3):
///
/// - **Major Type 0/1 (Unsigned/Negative Integers)**: No length concept - the value IS
///   the data
/// - **Major Type 2 (Byte String)**: Length indicates number of bytes in the string
/// - **Major Type 3 (Text String)**: Length indicates number of bytes in UTF-8 encoding
/// - **Major Type 4 (Array)**: Length indicates number of data items (elements) in the
///   array
/// - **Major Type 5 (Map)**: Length indicates number of key-value pairs in the map
/// - **Major Type 6 (Semantic Tag)**: Tags the following data item, length from tagged
///   content
/// - **Major Type 7 (Primitives)**: No length for simple values, floats, etc.
///
/// ## Errors
pub fn get_declared_length(bytes: &[u8]) -> Result<Option<usize>, minicbor::decode::Error> {
    let mut decoder = minicbor::Decoder::new(bytes);

    // Extract major type from high 3 bits of initial byte (RFC 8949 Section 3.1)
    match bytes.first().map(|&b| b >> 5) {
        Some(7 | 0 | 1 | 4 | 5 | 6) => Ok(None),
        Some(2) => {
            // Read length for byte string header
            let len = decoder.bytes()?;
            Ok(Some(len.len()))
        },
        Some(3) => {
            // Read length for text string header
            let len = decoder.str()?;
            Ok(Some(len.len()))
        },

        _ => Err(minicbor::decode::Error::message("Invalid type")),
    }
}

/// Returns the size of the CBOR header in bytes, based on RFC 8949 encoding rules.
///
/// CBOR encodes data items with a header that consists of:
/// 1. An initial byte containing:
///    - Major type (3 most significant bits)
///    - Additional information (5 least significant bits)
/// 2. Optional following bytes based on the additional information value
///
/// This function calculates only the size of the header itself, not including
/// any data that follows the header. It works with all CBOR major types:
/// - 0: Unsigned integer
/// - 1: Negative integer
/// - 2: Byte string
/// - 3: Text string
/// - 4: Array
/// - 5: Map
/// - 6: Tag
/// - 7: Simple/floating-point values
///
/// For deterministically encoded CBOR (as specified in RFC 8949 Section 4.2),
/// indefinite length items are not allowed, so this function will return an error
/// when encountering additional information value 31.
///
/// # Arguments
/// * `bytes` - A byte slice containing CBOR-encoded data
///
/// # Returns
/// * `Ok(usize)` - The size of the CBOR header in bytes
/// * `Err(DeterministicError)` - If the input is invalid or uses indefinite length
///   encoding
///
/// # Errors
/// Returns an error if:
/// - The input is empty
/// - The input uses indefinite length encoding (additional info = 31)
/// - The additional information value is invalid
pub fn get_cbor_header_size(bytes: &[u8]) -> Result<usize, minicbor::decode::Error> {
    // Extract the first byte which contains both major type and additional info
    let first_byte = bytes
        .first()
        .copied()
        .ok_or_else(|| minicbor::decode::Error::message("Empty cbor data"))?;
    // Major type is in the high 3 bits (not used in this function but noted for clarity)
    // let major_type = first_byte >> 5;
    // Additional info is in the low 5 bits and determines header size
    let additional_info = first_byte & 0b0001_1111;

    // Calculate header size based on additional info value
    match additional_info {
        // Values 0-23 are encoded directly in the additional info bits
        // Header is just the initial byte
        0..=23 => Ok(1),

        // Value 24 means the actual value is in the next 1 byte
        // Header is 2 bytes (initial byte + 1 byte)
        24 => Ok(2),

        // Value 25 means the actual value is in the next 2 bytes
        // Header is 3 bytes (initial byte + 2 bytes)
        25 => Ok(3),

        // Value 26 means the actual value is in the next 4 bytes
        // Header is 5 bytes (initial byte + 4 bytes)
        26 => Ok(5),

        // Value 27 means the actual value is in the next 8 bytes
        // Header is 9 bytes (initial byte + 8 bytes)
        27 => Ok(9),
        // Value 31 indicates indefinite length, which is not allowed in
        // deterministic encoding per RFC 8949 section 4.2.1
        31 => {
            Err(minicbor::decode::Error::message(
                "Cannot determine size of indefinite length item",
            ))
        },

        // Values 28-30 are reserved in RFC 8949 and not valid in current CBOR
        _ => {
            Err(minicbor::decode::Error::message(
                "Invalid additional info in CBOR header",
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `get_declared_length` for all CBOR major types per RFC 8949
    #[test]
    fn test_get_declared_length() {
        // Example 1: Empty byte string
        // Encoding: [0x40]
        // - 0x40 = 0b010_00000 (major type 2, additional info 0)
        // - Length: 0 bytes
        // - Content: none
        let empty_bytes = vec![0x40];

        let declared_length = get_declared_length(&empty_bytes).unwrap().unwrap();

        assert_eq!(declared_length, 0);

        // Example 2: 2-byte string with immediate length encoding
        // Encoding: [0x42, 0x01, 0x02]
        // - 0x42 = 0b010_00010 (major type 2, additional info 2)
        // - Length: 2 bytes (encoded immediately in additional info)
        // - Content: [0x01, 0x02]
        let short_bytes = vec![0x42, 0x01, 0x02];

        let declared_length = get_declared_length(&short_bytes).unwrap().unwrap();

        assert_eq!(declared_length, 2);

        // Example 3: 24-byte string requiring uint8 length encoding
        // Encoding: [0x58, 0x18, 0x01, 0x02, ..., 0x18]
        // - 0x58 = 0b010_11000 (major type 2, additional info 24)
        // - Length: 24 (encoded as uint8 in next byte: 0x18 = 24)
        // - Content: 24 bytes [0x01, 0x02, ..., 0x18]
        let mut medium_bytes = vec![0x58, 0x18]; // Header: byte string, uint8 length 24
        medium_bytes.extend((1..=24).collect::<Vec<u8>>()); // Content: 24 bytes

        let declared_length = get_declared_length(&medium_bytes).unwrap().unwrap();
        assert_eq!(declared_length, 24);

        // Example 4: 256-byte string requiring uint16 length encoding
        // Encoding: [0x59, 0x01, 0x00, 0x00, 0x00, ..., 0xFF]
        // - 0x59 = 0b010_11001 (major type 2, additional info 25)
        // - Length: 256 (encoded as uint16 big-endian: [0x01, 0x00])
        // - Content: 256 bytes [0x00, 0x00, ..., 0xFF]
        let mut large_bytes = vec![0x59, 0x01, 0x00]; // Header: byte string, uint16 length 256
        large_bytes.extend(vec![0x00; 256]); // Content: 256 zero bytes

        let declared_length = get_declared_length(&large_bytes).unwrap().unwrap();
        assert_eq!(declared_length, 256);
    }

    #[test]
    fn test_get_cbor_header_size() {
        // Test direct values (additional info 0-23)
        assert_eq!(get_cbor_header_size(&[0b000_00000]).unwrap(), 1); // Major type 0, value 0
        assert_eq!(get_cbor_header_size(&[0b001_10111]).unwrap(), 1); // Major type 1, value 23

        // Test 1-byte uint (additional info 24)
        assert_eq!(get_cbor_header_size(&[0b010_11000, 0x42]).unwrap(), 2); // Major type 2

        // Test 2-byte uint (additional info 25)
        assert_eq!(get_cbor_header_size(&[0b011_11001, 0x12, 0x34]).unwrap(), 3); // Major type 3

        // Test 4-byte uint (additional info 26)
        assert_eq!(
            get_cbor_header_size(&[0b100_11010, 0x12, 0x34, 0x56, 0x78]).unwrap(),
            5
        ); // Major type 4

        // Test 8-byte uint (additional info 27)
        assert_eq!(
            get_cbor_header_size(&[0b101_11011, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF])
                .unwrap(),
            9
        ); // Major type 5

        // Error cases
        // Empty input
        assert!(get_cbor_header_size(&[]).is_err());

        // Indefinite length (additional info 31)
        let result = get_cbor_header_size(&[0b110_11111]);
        assert!(result.is_err());

        // Small map (size 1) - additional info 1
        assert_eq!(get_cbor_header_size(&[0b101_00001]).unwrap(), 1); // Map with 1 pair

        // Large map (size 65535) - additional info 25 (2-byte uint follows)
        assert_eq!(get_cbor_header_size(&[0b101_11001, 0xFF, 0xFF]).unwrap(), 3); // Map with 65535 pairs

        // Reserved values (additional info 28-30)
        assert!(get_cbor_header_size(&[0b111_11100]).is_err()); // Major type 7, value 28
    }
}
