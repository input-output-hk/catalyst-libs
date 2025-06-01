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

use std::fmt;

use minicbor::Decoder;

/// CBOR Major Type for Arrays (4) shifted left by 5 bits
const CBOR_ARRAY_TYPE: u8 = 4 << 5;

/// CBOR header byte for indefinite-length arrays (major type 4, additional info 31)
const CBOR_INDEFINITE_ARRAY: u8 = CBOR_ARRAY_TYPE | 31;

/// CBOR array headers for different length encodings
const CBOR_ARRAY_UINT8: u8 = CBOR_ARRAY_TYPE | 24; // 0x98
const CBOR_ARRAY_UINT16: u8 = CBOR_ARRAY_TYPE | 25; // 0x99
const CBOR_ARRAY_UINT32: u8 = CBOR_ARRAY_TYPE | 26; // 0x9A
const CBOR_ARRAY_UINT64: u8 = CBOR_ARRAY_TYPE | 27; // 0x9B

/// Maximum values for each compact representation
const MAX_VALUE_UINT8: u64 = 23;
const MAX_VALUE_UINT16: u64 = u8::MAX as u64;
const MAX_VALUE_UINT32: u64 = u16::MAX as u64;
const MAX_VALUE_UINT64: u64 = u32::MAX as u64;

/// Error types that can occur during CBOR deterministic decoding validation.
///
/// These errors indicate violations of the deterministic encoding rules
/// as specified in RFC 8949 Section 4.2.
///
/// From RFC 8949:
/// "A CBOR item (data item) is determined to be encoded in a deterministic way if:"
/// - It follows minimal encoding rules for integers
/// - It contains no indefinite-length items
/// - All contained maps are ordered by their keys (lexicographically)
/// - No duplicate keys exist in maps
/// - All floating-point values use minimal length encoding
#[derive(Debug)]
pub enum DeterministicError {
    /// Indicates an integer is not encoded in its shortest possible representation.
    /// Per RFC 8949 Section 4.2.1:
    /// "An integer is encoded in the shortest form that can represent the value"
    NonMinimalInt,

    /// Indicates presence of indefinite-length items, which are forbidden.
    /// Per RFC 8949 Section 4.2.2:
    /// "Indefinite-length items must be made definite-length items"
    IndefiniteLength,

    /// Wraps decoding errors from the underlying CBOR decoder
    DecoderError(minicbor::decode::Error),

    /// Indicates map keys are not properly sorted.
    /// Per RFC 8949 Section 4.2.3:
    /// "The keys in every map must be sorted..."
    UnorderedMapKeys,

    /// Indicates a map contains duplicate keys.
    /// Per RFC 8949 Section 4.2.3:
    /// "No two keys in a map may be equal"
    DuplicateMapKey,

    /// Indicates float is not encoded in its shortest possible form.
    /// Per RFC 8949 Section 4.2.4:
    /// "Floating-point values must use the shortest form that preserves value"
    NonMinimalFloat,

    /// Indicates presence of non-finite floating point values.
    /// Per RFC 8949 Section 4.2.4:
    /// "Non-finite floating-point values (NaN, infinity, -infinity) are not allowed"
    NonFiniteFloat,

    /// Indicates unexpected end of input
    UnexpectedEof,

    /// Indicates an error occurred while decoding an array element
    ArrayElementDecoding,
}

impl fmt::Display for DeterministicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeterministicError::NonMinimalInt => write!(f, "integer not encoded in minimal form"),
            DeterministicError::IndefiniteLength => {
                write!(f, "indefinite-length items not allowed")
            },
            DeterministicError::DecoderError(e) => write!(f, "decoder error: {}", e),
            DeterministicError::UnorderedMapKeys => write!(f, "map keys not in canonical order"),
            DeterministicError::DuplicateMapKey => write!(f, "duplicate map key found"),
            DeterministicError::NonMinimalFloat => write!(f, "float not encoded in minimal form"),
            DeterministicError::NonFiniteFloat => write!(f, "non-finite float values not allowed"),
            DeterministicError::UnexpectedEof => write!(f, "unexpected end of input"),
            DeterministicError::ArrayElementDecoding => write!(f, "error decoding array element"),
        }
    }
}

impl std::error::Error for DeterministicError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DeterministicError::DecoderError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<minicbor::decode::Error> for DeterministicError {
    fn from(error: minicbor::decode::Error) -> Self {
        DeterministicError::DecoderError(error)
    }
}

/// Validates that a CBOR array length is encoded using the minimal number of bytes
/// as required by RFC 8949 Section 4.2.1.
///
/// According to the RFC:
/// - An integer MUST be represented in the smallest possible encoding
/// - For example:
///   - 0 through 23 MUST be expressed in the simple value form
///   - 24 through 255 MUST be expressed only as uint8
///   - 256 through 65535 MUST be expressed only as uint16
///   - 65536 through 4294967295 MUST be expressed only as uint32
///   - 4294967296 through 18446744073709551615 MUST be expressed only as uint64
///
/// # Arguments
/// * `d` - The CBOR decoder containing the input data
/// * `start_pos` - Starting position of the array header in the input
/// * `length` - The decoded array length value
///
/// # Returns
/// * `Ok(())` if the length is encoded minimally
/// * `Err(DeterministicError::NonMinimalInt)` if the length could have been encoded in a
///   shorter form
fn check_minimal_length(
    d: &Decoder, start_pos: usize, length: u64,
) -> Result<(), DeterministicError> {
    // Get the initial byte which indicates the encoding type used
    let initial_byte = d.input()[start_pos];

    match initial_byte {
        // If encoded as uint8 (1 byte, additional info 24)
        // RFC 8949: "The value 24 MUST be used only if the value cannot be expressed using the
        // simple value"
        CBOR_ARRAY_UINT8 => {
            if length <= MAX_VALUE_UINT8 {
                // Error if value could have fit in 5-bit immediate value (0-23)
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint16 (2 bytes, additional info 25)
        // RFC 8949: "The value 25 MUST be used only if the value cannot be expressed using the
        // simple value or uint8"
        CBOR_ARRAY_UINT16 => {
            if length <= MAX_VALUE_UINT16 {
                // Error if value could have fit in uint8
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint32 (4 bytes, additional info 26)
        // RFC 8949: "The value 26 MUST be used only if the value cannot be expressed using the
        // simple value, uint8, or uint16"
        CBOR_ARRAY_UINT32 => {
            if length <= MAX_VALUE_UINT32 {
                // Error if value could have fit in uint16
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint64 (8 bytes, additional info 27)
        // RFC 8949: "The value 27 MUST be used only if the value cannot be expressed using the
        // simple value, uint8, uint16, or uint32"
        CBOR_ARRAY_UINT64 => {
            if length <= MAX_VALUE_UINT64 {
                // Error if value could have fit in uint32
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // For immediate values (0-23), no minimality check is needed
        // as these are already the most compact form possible
        _ => {},
    }

    Ok(())
}

/// Decodes a CBOR array with deterministic encoding validation (RFC 8949 Section 4.2)
///
/// # Errors
///
/// Returns `DeterministicError` if:
/// - Input is empty (UnexpectedEof)
/// - Array uses indefinite-length encoding (IndefiniteLength)
/// - Array length is not encoded minimally (NonMinimalInt)
/// - Array element decoding fails (ArrayElementDecoding)
pub fn decode_array_deterministcally(d: &mut Decoder) -> Result<Vec<u8>, DeterministicError> {
    validate_input_not_empty(d)?;
    validate_not_indefinite_length(d)?;

    let start_pos = d.position();
    let array_length = d.array()?;

    match array_length {
        None => Ok(Vec::new()),
        Some(length) => {
            check_minimal_length(d, start_pos, length)?;
            decode_array_elements(d, length)
        },
    }
}

/// Ensures the decoder has remaining input data
fn validate_input_not_empty(d: &Decoder) -> Result<(), DeterministicError> {
    if d.position() >= d.input().len() {
        return Err(DeterministicError::UnexpectedEof);
    }
    Ok(())
}

/// Checks if array uses indefinite-length encoding (forbidden by RFC 8949)
fn validate_not_indefinite_length(d: &Decoder) -> Result<(), DeterministicError> {
    let initial_byte = d.input()[d.position()];
    if initial_byte == CBOR_INDEFINITE_ARRAY {
        return Err(DeterministicError::IndefiniteLength);
    }
    Ok(())
}

/// Decodes array elements into a vector
fn decode_array_elements(d: &mut Decoder, length: u64) -> Result<Vec<u8>, DeterministicError> {
    let mut result = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let element = d
            .decode()
            .map_err(|_| DeterministicError::ArrayElementDecoding)?;
        result.push(element);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_decoder(bytes: &[u8]) -> Decoder<'_> {
        Decoder::new(bytes)
    }

    /// Test handling of empty input.
    /// While not explicitly mentioned in RFC 8949, proper handling of malformed
    /// input is essential for robust CBOR processing.
    #[test]
    fn test_empty_input() {
        let empty_bytes: &[u8] = &[];
        let mut decoder = create_decoder(empty_bytes);
        assert!(matches!(
            decode_array_deterministcally(&mut decoder),
            Err(DeterministicError::UnexpectedEof)
        ));
    }

    /// Test array length encoding rules.
    ///
    /// RFC 8949 Section 4.2.1 states:
    /// "Integers must be as small as possible."
    /// "What this means is that the shortest form of encoding must be used,
    /// in particular:
    /// - 0 to 23 and -1 to -24 must be expressed in the same byte as the major type;
    /// - 24 to 255 and -25 to -256 must be expressed only with an additional uint8_t;
    /// - 256 to 65535 and -257 to -65536 must be expressed only with an additional
    ///   uint16_t"
    #[test]
    fn test_minimal_length_array() {
        // Test case 1: Array of length 5 encoded minimally
        // 0x85 = array (major type 4) with length 5 directly encoded
        let valid_bytes = [0x85, 0x01, 0x02, 0x03, 0x04, 0x05];
        let mut decoder = create_decoder(&valid_bytes);
        assert!(decode_array_deterministcally(&mut decoder).is_ok());

        // Test case 2: Same array with non-minimal encoding
        // 0x98 0x05 = array with length 5 encoded in additional byte (non-minimal)
        let invalid_bytes = [0x98, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
        let mut decoder = create_decoder(&invalid_bytes);
        assert!(matches!(
            decode_array_deterministcally(&mut decoder),
            Err(DeterministicError::NonMinimalInt)
        ));
    }

    /// Test indefinite length array detection.
    ///
    /// RFC 8949 Section 4.2.2 states:
    /// "Indefinite-length items must be made definite-length items.
    /// - The implementations must not generate indefinite-length items
    /// - The implementations must support parsing indefinite-length items"
    #[test]
    fn test_indefinite_length_array() {
        // 0x9F = indefinite-length array start
        // 0xFF = "break" stop code
        let invalid_bytes = [0x9F, 0x01, 0x02, 0x03, 0xFF];
        let mut decoder = create_decoder(&invalid_bytes);
        assert!(matches!(
            decode_array_deterministcally(&mut decoder),
            Err(DeterministicError::IndefiniteLength)
        ));
    }

    /// Test handling of malformed array elements.
    /// While not explicitly covered in RFC 8949's deterministic encoding section,
    /// proper error handling of malformed data is essential.
    #[test]
    fn test_array_element_decoding_error() {
        // 0x81 = array of length 1
        // 0xFF = invalid element (break code outside indefinite-length context)
        let invalid_bytes = [0x81, 0xFF];
        let mut decoder = create_decoder(&invalid_bytes);
        assert!(matches!(
            decode_array_deterministcally(&mut decoder),
            Err(DeterministicError::ArrayElementDecoding)
        ));
    }

    /// Test array length encoding at boundary conditions.
    ///
    /// RFC 8949 Section 4.2.1 requires different encoding sizes based on value ranges.
    /// This test verifies proper handling of boundary conditions for various length
    /// ranges.
    #[test]
    fn test_length_encoding_boundaries() {
        // Test case 1: Maximum direct value (23)
        let valid_23 = [0x97]; // length 23 encoded directly
        let mut decoder = create_decoder(&valid_23);
        assert!(check_minimal_length(&mut decoder, 0, 23).is_ok());

        // Test case 2: Minimum one-byte value (24)
        let valid_24 = [0x98, 24]; // length 24 encoded with one additional byte
        let mut decoder = create_decoder(&valid_24);
        assert!(check_minimal_length(&mut decoder, 0, 24).is_ok());

        // Test case 3: Maximum one-byte value (255)
        let valid_255 = [0x98, 0xFF]; // length 255 encoded with one additional byte
        let mut decoder = create_decoder(&valid_255);
        assert!(check_minimal_length(&mut decoder, 0, 255).is_ok());

        // Test case 4: Minimum two-byte value (256)
        let valid_256 = [0x99, 0x01, 0x00]; // length 256 encoded with two additional bytes
        let mut decoder = create_decoder(&valid_256);
        assert!(check_minimal_length(&mut decoder, 0, 256).is_ok());
    }

    /// Test non-minimal length encoding detection.
    ///
    /// RFC 8949 Section 4.2.1 requires that "The number of bytes used to specify
    /// a length must be as small as possible"
    #[test]
    fn test_non_minimal_length_detection() {
        // Test case 1: Small value (5) encoded with one byte
        let invalid_small = [0x98, 0x05]; // length 5 encoded with additional byte
        let mut decoder = create_decoder(&invalid_small);
        assert!(matches!(
            check_minimal_length(&mut decoder, 0, 5),
            Err(DeterministicError::NonMinimalInt)
        ));

        // Test case 2: Value 24 encoded with two bytes
        let invalid_24 = [0x99, 0x00, 0x18]; // length 24 encoded with two bytes
        let mut decoder = create_decoder(&invalid_24);
        assert!(matches!(
            check_minimal_length(&mut decoder, 0, 24),
            Err(DeterministicError::NonMinimalInt)
        ));
    }

    /// Test handling of arrays with various sizes.
    ///
    /// This ensures compliance with RFC 8949 Section 4.2.1's requirements
    /// for minimal encoding while handling different array sizes correctly.
    #[test]
    fn test_array_size_variants() {
        // Test case 1: Empty array (length 0)
        let empty_array = [0x80]; // array of length 0
        let mut decoder = create_decoder(&empty_array);
        assert!(decode_array_deterministcally(&mut decoder).is_ok());

        // Test case 2: Single element array
        let single_element = [0x81, 0x01];
        let mut decoder = create_decoder(&single_element);
        assert!(decode_array_deterministcally(&mut decoder).is_ok());

        // Test case 3: Array with 23 elements (maximum direct encoding)
        let mut max_direct = vec![0x97]; // array of length 23
        max_direct.extend(vec![0x01; 23]);
        let mut decoder = create_decoder(&max_direct);
        assert!(decode_array_deterministcally(&mut decoder).is_ok());
    }
}
