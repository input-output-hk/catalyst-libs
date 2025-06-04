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

use std::{cmp::Ordering, fmt};

use minicbor::Decoder;

/// Major type indicator for CBOR maps (major type 5: 101 in top 3 bits)
/// As per RFC 8949 Section 4.2.3, maps in deterministic encoding must:
/// - Have keys sorted by length first, then bytewise lexicographically
/// - Contain no duplicate keys
const CBOR_MAJOR_TYPE_MAP: u8 = 5 << 5;

/// Major type indicator for CBOR arrays (major type 4: 100 in top 3 bits)
/// As per RFC 8949 Section 4.2.1, array lengths must use the smallest possible encoding
const CBOR_MAJOR_TYPE_ARRAY: u8 = 4 << 5;

/// Major type indicator for CBOR text strings (major type 3: 011 in top 3 bits)
/// Text strings in deterministic encoding must:
/// - Use definite lengths (no chunking) per RFC 8949 Section 4.2.2
/// - Use the smallest possible length encoding per Section 4.2.1
const CBOR_MAJOR_TYPE_TEXT_STRING: u8 = 3 << 5;

/// Major type indicator for CBOR byte strings (major type 2: 010 in top 3 bits)
/// Byte strings in deterministic encoding must:
/// - Use definite lengths (no chunking) per RFC 8949 Section 4.2.2
/// - Use the smallest possible length encoding per Section 4.2.1
const CBOR_MAJOR_TYPE_BYTE_STRING: u8 = 2 << 5;

/// Indicator for indefinite-length maps (major type 5 with additional info 31)
/// RFC 8949 Section 4.2.2: "Indefinite-length items must be made definite-length items."
/// This value is used to detect and reject indefinite-length maps in deterministic
/// encoding.
const CBOR_INDEFINITE_LENGTH_MAP: u8 = CBOR_MAJOR_TYPE_MAP | 31;

/// Indicator for indefinite-length text strings (major type 3 with additional info 31)
/// RFC 8949 Section 4.2.2: "Indefinite-length items must be made definite-length items."
/// This value is used to detect and reject indefinite-length text strings in
/// deterministic encoding.
const CBOR_INDEFINITE_LENGTH_TEXT: u8 = CBOR_MAJOR_TYPE_TEXT_STRING | 31;

/// Indicator for indefinite-length byte strings (major type 2 with additional info 31)
/// RFC 8949 Section 4.2.2: "Indefinite-length items must be made definite-length items."
/// This value is used to detect and reject indefinite-length byte strings in
/// deterministic encoding.
const CBOR_INDEFINITE_LENGTH_BYTES: u8 = CBOR_MAJOR_TYPE_BYTE_STRING | 31;

/// Indicator for array length encoded as uint8 (major type 4 with additional info 24)
/// RFC 8949 Section 4.2.1: "24 to 255 must be expressed only with an additional
/// `uint8_t`"
const CBOR_ARRAY_LENGTH_UINT8: u8 = CBOR_MAJOR_TYPE_ARRAY | 24;

/// Indicator for array length encoded as uint16 (major type 4 with additional info 25)
/// RFC 8949 Section 4.2.1: "256 to 65535 must be expressed only with an additional
/// `uint16_t`"
const CBOR_ARRAY_LENGTH_UINT16: u8 = CBOR_MAJOR_TYPE_ARRAY | 25;

/// Indicator for array length encoded as uint32 (major type 4 with additional info 26)
/// RFC 8949 Section 4.2.1: "65536 to 4294967295 must be expressed only with an additional
/// `uint32_t`"
const CBOR_ARRAY_LENGTH_UINT32: u8 = CBOR_MAJOR_TYPE_ARRAY | 26;

/// Indicator for array length encoded as uint64 (major type 4 with additional info 27)
/// RFC 8949 Section 4.2.1: "4294967296 to 18446744073709551615 must be expressed only
/// with an additional `uint64_t`"
const CBOR_ARRAY_LENGTH_UINT64: u8 = CBOR_MAJOR_TYPE_ARRAY | 27;

/// Additional info value for string length encoded as uint8 (24)
/// RFC 8949 Section 4.2.1: "The value 24 MUST be used only if the value cannot be
/// expressed using the simple value" Used for lengths 24 to 255
const CBOR_STRING_LENGTH_UINT8: u8 = 24;

/// Additional info value for string length encoded as uint16 (25)
/// RFC 8949 Section 4.2.1: "The value 25 MUST be used only if the value cannot be
/// expressed using ... uint8" Used for lengths 256 to 65535
const CBOR_STRING_LENGTH_UINT16: u8 = 25;

/// Additional info value for string length encoded as uint32 (26)
/// RFC 8949 Section 4.2.1: "The value 26 MUST be used only if the value cannot be
/// expressed using ... uint16" Used for lengths 65536 to 4294967295
const CBOR_STRING_LENGTH_UINT32: u8 = 26;

/// Additional info value for string length encoded as uint64 (27)
/// RFC 8949 Section 4.2.1: "The value 27 MUST be used only if the value cannot be
/// expressed using ... uint32" Used for lengths 4294967296 to 18446744073709551615
const CBOR_STRING_LENGTH_UINT64: u8 = 27;

/// Maximum value that can be encoded in a 5-bit additional info field
/// RFC 8949 Section 4.2.1: "0 to 23 must be expressed in the same byte as the major type"
/// Values 0-23 are encoded directly in the additional info field of the initial byte
const CBOR_MAX_TINY_VALUE: u64 = 23;

/// Maximum value that can be encoded in a uint8 additional info field
/// RFC 8949 Section 4.2.1: "24 to 255 must be expressed only with an additional
/// `uint8_t`"
const CBOR_MAX_UINT8_VALUE: u64 = u8::MAX as u64;

/// Maximum value that can be encoded in a uint16 additional info field
/// RFC 8949 Section 4.2.1: "256 to 65535 must be expressed only with an additional
/// `uint16_t`"
const CBOR_MAX_UINT16_VALUE: u64 = u16::MAX as u64;

/// Maximum value that can be encoded in a uint32 additional info field
/// RFC 8949 Section 4.2.1: "65536 to 4294967295 must be expressed only with an additional
/// `uint32_t`"
const CBOR_MAX_UINT32_VALUE: u64 = u32::MAX as u64;

const CBOR_MAP_LENGTH_UINT8: u8 = CBOR_MAJOR_TYPE_MAP | 24; // For uint8 length encoding

const MAP_TYPE_MASK: u8 = 0xE0; // Mask for extracting major type (top 3 bits)

/// A configurable CBOR decoder that can operate in either deterministic or
/// non-deterministic mode
pub struct CborDecoder {
    deterministic: bool,
}

impl CborDecoder {
    /// Creates a new CborDecoder with specified deterministic mode
    pub fn new(deterministic: bool) -> Self {
        Self { deterministic }
    }

    /// Decodes CBOR data from the given decoder
    /// Decodes CBOR data with optional deterministic validation.
    /// Returns the raw bytes of the decoded data.
    pub fn decode(&self, decoder: &mut Decoder<'_>) -> Result<Vec<u8>, DeterministicError> {
        let start_position = decoder.position();

        // Early return if input is empty
        let first_byte = decoder
            .input()
            .get(start_position)
            .ok_or(DeterministicError::UnexpectedEof)?;

        // Check if we need deterministic map decoding
        let is_map_type = (first_byte & MAP_TYPE_MASK) == CBOR_MAJOR_TYPE_MAP;
        if is_map_type && self.deterministic {
            return decode_map_deterministically(decoder);
        }

        // For non-map types or non-deterministic decoding, skip and return raw bytes
        decoder.skip()?;
        let decoded_bytes = self.extract_decoded_bytes(decoder, start_position);
        Ok(decoded_bytes)
    }

    /// Extracts the decoded bytes from the decoder's input
    #[inline]
    fn extract_decoded_bytes(&self, decoder: &Decoder<'_>, start_pos: usize) -> Vec<u8> {
        decoder.input()[start_pos..decoder.position()].to_vec()
    }
}

/// Represents a CBOR map key-value pair where the key must be deterministically encoded
/// according to RFC 8949 Section 4.2.3.
///
/// This type stores the raw bytes of both key and value to enable:
/// 1. Length-first ordering of keys (shorter keys before longer ones)
/// 2. Lexicographic comparison of equal-length keys
/// 3. Preservation of the original encoded form
#[derive(Clone)]
pub struct MapEntry {
    /// Raw bytes of the encoded key, used for deterministic ordering
    pub key_bytes: Vec<u8>,
    /// Raw bytes of the encoded value
    pub value: Vec<u8>,
}

impl MapEntry {
    /// Compare map entries according to RFC 8949 Section 4.2.3 rules:
    /// 1. Compare by length of encoded key
    /// 2. If lengths equal, compare bytewise lexicographically
    fn compare(&self, other: &Self) -> Ordering {
        match self.key_bytes.len().cmp(&other.key_bytes.len()) {
            Ordering::Equal => self.key_bytes.cmp(&other.key_bytes),
            ordering => ordering,
        }
    }
}

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
            DeterministicError::DecoderError(e) => write!(f, "decoder error: {e}"),
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

/// Decodes a CBOR map with deterministic encoding validation (RFC 8949 Section 4.2.3)
/// Returns the raw bytes of the map if it passes all deterministic validation rules.
///
/// From RFC 8949 Section 4.2.3:
/// "The keys in every map must be sorted in the following order:
///  1. If two keys have different lengths, the shorter one sorts earlier;
///  2. If two keys have the same length, the one with the lower value in (byte-wise)
///     lexical order sorts earlier."
///
/// Additionally:
/// - Map lengths must use minimal encoding (Section 4.2.1)
/// - Indefinite-length maps are not allowed (Section 4.2.2)
/// - No two keys may be equal (Section 4.2.3)
/// - The keys themselves must be deterministically encoded
///
/// # Errors
///
/// Returns `DeterministicError` if:
/// - Input is empty (`UnexpectedEof`)
/// - Map uses indefinite-length encoding (`IndefiniteLength`)
/// - Map length is not encoded minimally (`NonMinimalInt`)
/// - Map keys are not properly sorted (`UnorderedMapKeys`)
/// - Duplicate keys are found (`DuplicateMapKey`)
/// - Map key or value decoding fails (`DecoderError`)
pub fn decode_map_deterministically(d: &mut Decoder) -> Result<Vec<u8>, DeterministicError> {
    validate_input_not_empty(d)?;
    validate_not_indefinite_length_map(d)?;

    let start_pos = d.position();
    let map_len = d.map()?.ok_or(DeterministicError::UnexpectedEof)?;

    check_minimal_length(d, start_pos, map_len)?;

    // Store the starting position of the entire map
    let map_start = start_pos;

    // Decode entries to validate them
    let entries = decode_map_entries(d, map_len)?;
    validate_map_ordering(&entries)?;

    // Get the ending position after validation
    let map_end = d.position();

    // Return the raw bytes of the entire validated map
    Ok(d.input()[map_start..map_end].to_vec())
}

/// Validates that map does not use indefinite-length encoding
fn validate_not_indefinite_length_map(d: &Decoder) -> Result<(), DeterministicError> {
    let initial_byte = d.input()[d.position()];
    if initial_byte == CBOR_INDEFINITE_LENGTH_MAP {
        return Err(DeterministicError::IndefiniteLength);
    }
    Ok(())
}

/// Decodes all key-value pairs in the map
fn decode_map_entries(d: &mut Decoder, length: u64) -> Result<Vec<MapEntry>, DeterministicError> {
    let mut entries = Vec::with_capacity(length as usize);

    for _ in 0..length {
        // Validate and decode key
        let key_start = d.position();
        validate_string_length(d, key_start)?; // Add string length validation
        d.skip()?;
        let key_end = d.position();
        let key_bytes = d.input()[key_start..key_end].to_vec();

        // Validate and decode value
        let value_start = d.position();
        validate_string_length(d, value_start)?; // Add string length validation
        d.skip()?;
        let value_end = d.position();
        let value = d.input()[value_start..value_end].to_vec();

        entries.push(MapEntry { key_bytes, value });
    }

    Ok(entries)
}

/// Validates map keys are properly ordered according to RFC 8949 Section 4.2.3
/// and checks for duplicate keys
fn validate_map_ordering(entries: &[MapEntry]) -> Result<(), DeterministicError> {
    if entries.is_empty() {
        return Ok(());
    }

    for i in 0..entries.len() - 1 {
        match entries[i].compare(&entries[i + 1]) {
            Ordering::Equal => return Err(DeterministicError::DuplicateMapKey),
            Ordering::Greater => return Err(DeterministicError::UnorderedMapKeys),
            Ordering::Less => continue,
        }
    }

    Ok(())
}

/// Ensures the decoder has remaining input data
fn validate_input_not_empty(d: &Decoder) -> Result<(), DeterministicError> {
    if d.position() >= d.input().len() {
        return Err(DeterministicError::UnexpectedEof);
    }
    Ok(())
}

/// Validates that a CBOR array length is encoded using the minimal number of bytes
/// as required by RFC 8949 Section 4.2.1.
///
/// From RFC 8949 Section 4.2.1:
/// "Integers must be as small as possible. What this means is that the shortest
/// form of encoding must be used, in particular:
/// - 0 to 23 must be expressed in the same byte as the major type
/// - 24 to 255 must be expressed only with an additional `uint8_t`
/// - 256 to 65535 must be expressed only with an additional `uint16_t`
/// - 65536 to 4294967295 must be expressed only with an additional `uint32_t`
/// - 4294967296 to 18446744073709551615 must be expressed only with an additional
///   `uint64_t`"
fn check_minimal_length(
    d: &Decoder, start_pos: usize, length: u64,
) -> Result<(), DeterministicError> {
    // Get the initial byte which indicates the encoding type used
    let initial_byte = d.input()[start_pos];

    // Extract major type (top 3 bits)
    let _major_type = initial_byte & 0xE0; // 0xe0 = 11100000

    match initial_byte {
        // Check both array and map uint8 length encodings
        b if b == CBOR_ARRAY_LENGTH_UINT8 || b == CBOR_MAP_LENGTH_UINT8 => {
            if length <= CBOR_MAX_TINY_VALUE {
                return Err(DeterministicError::NonMinimalInt);
            }
        }
,
        // If encoded as uint8 (1 byte, additional info 24)
        // RFC 8949: "The value 24 MUST be used only if the value cannot be expressed using the
        // simple value"
        CBOR_ARRAY_LENGTH_UINT8 => {
            if length <= CBOR_MAX_TINY_VALUE {
                // Error if value could have fit in 5-bit immediate value (0-23)
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint16 (2 bytes, additional info 25)
        // RFC 8949: "The value 25 MUST be used only if the value cannot be expressed using the
        // simple value or uint8"
        CBOR_ARRAY_LENGTH_UINT16 => {
            if length <= CBOR_MAX_UINT8_VALUE {
                // Error if value could have fit in uint8
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint32 (4 bytes, additional info 26)
        // RFC 8949: "The value 26 MUST be used only if the value cannot be expressed using the
        // simple value, uint8, or uint16"
        CBOR_ARRAY_LENGTH_UINT32 => {
            if length <= CBOR_MAX_UINT16_VALUE {
                // Error if value could have fit in uint16
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        // If encoded as uint64 (8 bytes, additional info 27)
        // RFC 8949: "The value 27 MUST be used only if the value cannot be expressed using the
        // simple value, uint8, uint16, or uint32"
        CBOR_ARRAY_LENGTH_UINT64 => {
            if length <= CBOR_MAX_UINT32_VALUE {
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

/// Validates that a CBOR string length is encoded using the minimal number of bytes
/// as required by RFC 8949 Section 4.2.1.
///
/// # Rules for minimal encoding:
/// - 0 to 23: must be expressed in the same byte as the major type
/// - 24 to 255: must use uint8
/// - 256 to 65535: must use uint16
/// - 65536 to 4294967295: must use uint32
/// - 4294967296 to 18446744073709551615: must use uint64
///
/// # Arguments
/// * `d` - The CBOR decoder containing the input
/// * `start_pos` - Starting position of the string in the input
///
/// # Returns
/// * `Ok(())` if the string length is encoded minimally
/// * `Err(DeterministicError)` if encoding is non-minimal or invalid
fn validate_string_length(d: &Decoder, start_pos: usize) -> Result<(), DeterministicError> {
    let input = d.input();

    // Check if we have at least one byte
    if start_pos >= input.len() {
        return Err(DeterministicError::UnexpectedEof);
    }

    let initial_byte = input[start_pos];

    // Early return if not a string type
    if !is_string_type(initial_byte) {
        return Ok(());
    }

    // Check for indefinite length strings (not allowed)
    if is_indefinite_string(initial_byte) {
        return Err(DeterministicError::IndefiniteLength);
    }

    let additional_info = initial_byte & 0x1F; // Extract additional info (bottom 5 bits)
    let length = decode_string_length(d, start_pos, additional_info)?;
    validate_length_minimality(length, additional_info)
}

/// Checks if the byte represents a CBOR string type (text or bytes)
#[inline]
fn is_string_type(byte: u8) -> bool {
    let major_type = byte & 0xE0; // Extract major type (top 3 bits)
    major_type == CBOR_MAJOR_TYPE_TEXT_STRING || major_type == CBOR_MAJOR_TYPE_BYTE_STRING
}

/// Checks if the byte represents an indefinite-length string
#[inline]
fn is_indefinite_string(byte: u8) -> bool {
    byte == CBOR_INDEFINITE_LENGTH_TEXT || byte == CBOR_INDEFINITE_LENGTH_BYTES
}

/// Ensures the input slice has enough bytes available starting from `start_pos`
#[inline]
fn check_slice_range(
    input: &[u8], start_pos: usize, additional_bytes: usize,
) -> Result<(), DeterministicError> {
    if start_pos
        .checked_add(additional_bytes)
        .is_none_or(|end| end > input.len())
    {
        return Err(DeterministicError::UnexpectedEof);
    }
    Ok(())
}

/// Gets a slice of the input with bounds checking
#[inline]
fn get_checked_slice(
    input: &[u8], start_pos: usize, length: usize,
) -> Result<&[u8], DeterministicError> {
    check_slice_range(input, start_pos, length)?;
    Ok(&input[start_pos..start_pos + length])
}

/// Decodes the string length based on the additional info value
fn decode_string_length(
    d: &Decoder, start_pos: usize, additional_info: u8,
) -> Result<u64, DeterministicError> {
    let input = d.input();

    match additional_info {
        0..=23 => Ok(u64::from(additional_info)), // Direct value

        CBOR_STRING_LENGTH_UINT8 => {
            let bytes = get_checked_slice(input, start_pos + 1, 1)?;
            Ok(u64::from(bytes[0]))
        },

        CBOR_STRING_LENGTH_UINT16 => {
            let bytes = get_checked_slice(input, start_pos + 1, 2)?;
            Ok(u64::from(u16::from_be_bytes(bytes.try_into().unwrap())))
        },

        CBOR_STRING_LENGTH_UINT32 => {
            let bytes = get_checked_slice(input, start_pos + 1, 4)?;
            Ok(u64::from(u32::from_be_bytes(bytes.try_into().unwrap())))
        },

        CBOR_STRING_LENGTH_UINT64 => {
            let bytes = get_checked_slice(input, start_pos + 1, 8)?;
            Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
        },

        _ => {
            Err(DeterministicError::DecoderError(
                minicbor::decode::Error::message("invalid additional info for string length"),
            ))
        },
    }
}

/// Validates that the length uses minimal encoding according to RFC 8949
fn validate_length_minimality(length: u64, encoding_used: u8) -> Result<(), DeterministicError> {
    match encoding_used {
        CBOR_STRING_LENGTH_UINT8 => {
            if length <= 23 {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_LENGTH_UINT16 => {
            if u8::try_from(length).is_ok() {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_LENGTH_UINT32 => {
            if u16::try_from(length).is_ok() {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_LENGTH_UINT64 => {
            if u32::try_from(length).is_ok() {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        _ => {}, // Direct values 0-23 are always minimal
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use minicbor::encode::Encoder;

    use super::*;

    /// Test the deterministic map validation rules from RFC 8949 Section 4.2.3.
    ///
    /// The RFC mandates:
    /// 1. Keys must be sorted by length first
    /// 2. Equal length keys must be sorted lexicographically
    /// 3. No duplicate keys are allowed
    #[test]
    fn test_map_validation() {
        // Test case 1: Valid ordering - shorter key before longer key
        let valid_map = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x02, // Key 1: 2-byte string
            0x41, 0x01, // Value 1: 1-byte string
            0x43, 0x01, 0x02, 0x03, // Key 2: 3-byte string
            0x41, 0x02, // Value 2: 1-byte string
        ];
        let mut decoder = Decoder::new(&valid_map);
        assert!(decode_map_deterministically(&mut decoder).is_ok());

        // Test case 2: Invalid ordering - longer key before shorter key
        let invalid_map = vec![
            0xA2, // Map with 2 pairs
            0x43, 0x01, 0x02, 0x03, // Key 1: 3-byte string (longer first - invalid)
            0x41, 0x01, // Value 1: 1-byte string
            0x42, 0x01, 0x02, // Key 2: 2-byte string
            0x41, 0x02, // Value 2: 1-byte string
        ];
        let mut decoder = Decoder::new(&invalid_map);
        assert!(matches!(
            decode_map_deterministically(&mut decoder),
            Err(DeterministicError::UnorderedMapKeys)
        ));

        // Test case 3: Duplicate keys
        let duplicate_map = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x02, // Key 1: 2-byte string
            0x41, 0x01, // Value 1: 1-byte string
            0x42, 0x01, 0x02, // Key 2: same as Key 1 (duplicate - invalid)
            0x41, 0x02, // Value 2: 1-byte string
        ];
        let mut decoder = Decoder::new(&duplicate_map);
        assert!(matches!(
            decode_map_deterministically(&mut decoder),
            Err(DeterministicError::DuplicateMapKey)
        ));
    }

    #[test]
    // Ensures that encoding and decoding a map preserves:
    /// - The bytewise lexicographic ordering of keys
    /// - The exact byte representation of values
    /// - The definite length encoding format
    fn test_map_bytes_roundtrip() {
        // Create a valid deterministic map encoding
        let valid_map = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x02, // Key 1: 2-byte string
            0x41, 0x01, // Value 1: 1-byte string
            0x43, 0x01, 0x02, 0x03, // Key 2: 3-byte string
            0x41, 0x02, // Value 2: 1-byte string
        ];

        let mut decoder = Decoder::new(&valid_map);
        let result = decode_map_deterministically(&mut decoder).unwrap();

        // Verify we got back exactly the same bytes
        assert_eq!(result, valid_map);
    }

    /// Verifies string length encoding follows RFC 8949 requirements:
    /// - Must use the shortest possible length encoding
    /// - Length must be encoded as definite length
    /// - No indefinite length strings allowed
    #[test]
    fn test_string_length_validation() {
        // Test case 1: Valid minimal encoding for small string
        let valid_small = vec![
            0x63, // Text string, length 3 (0x60 | 3)
            b'f', b'o', b'o',
        ];
        let decoder = Decoder::new(&valid_small);
        assert!(validate_string_length(&decoder, 0).is_ok());

        // Test case 2: Non-minimal encoding for small string
        let invalid_small = vec![
            0x78, 0x03, // Text string, length 3 with uint8 when not needed
            b'f', b'o', b'o',
        ];
        let decoder = Decoder::new(&invalid_small);
        assert!(matches!(
            validate_string_length(&decoder, 0),
            Err(DeterministicError::NonMinimalInt)
        ));

        // Test case 3: Valid encoding for medium string
        let mut valid_medium = vec![
            0x78, 0x80, // Text string, length 128 with uint8
        ];
        valid_medium.extend(vec![b'x'; 128]);
        let decoder = Decoder::new(&valid_medium);
        assert!(validate_string_length(&decoder, 0).is_ok());

        // Test case 4: Indefinite length string
        let indefinite = vec![
            0x7F, // Indefinite length text string
            0x63, b'f', b'o', b'o', 0xFF, // Break
        ];
        let decoder = Decoder::new(&indefinite);
        assert!(matches!(
            validate_string_length(&decoder, 0),
            Err(DeterministicError::IndefiniteLength)
        ));
    }

    /// Test cases for lexicographic ordering of map keys as specified in RFC 8949 Section
    /// 4.2.3.
    ///
    /// From RFC 8949 Section 4.2.3:
    /// "The keys in every map must be sorted in the following order:
    ///  1. If two keys have different lengths, the shorter one sorts earlier;
    ///  2. If two keys have the same length, the one with the lower value in (byte-wise)
    ///     lexical order sorts earlier."
    #[test]
    fn test_map_lexicographic_ordering() {
        // Test case: Equal length keys must be sorted lexicographically
        // This follows rule 2 from RFC 8949 Section 4.2.3 for same-length keys
        let valid_map = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x02, // Key 1: [0x01, 0x02]
            0x41, 0x01, // Value 1
            0x42, 0x01, 0x03, // Key 2: [0x01, 0x03] (lexicographically larger)
            0x41, 0x02, // Value 2
        ];
        let mut decoder = Decoder::new(&valid_map);
        assert!(decode_map_deterministically(&mut decoder).is_ok());

        // Invalid ordering - violates RFC 8949 Section 4.2.3 rule 2:
        // "If two keys have the same length, the one with the lower value in
        // (byte-wise) lexical order sorts earlier"
        let invalid_map = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x03, // Key 1: [0x01, 0x03]
            0x41, 0x01, // Value 1
            0x42, 0x01, 0x02, // Key 2: [0x01, 0x02] (should come first)
            0x41, 0x02, // Value 2
        ];
        let mut decoder = Decoder::new(&invalid_map);
        assert!(matches!(
            decode_map_deterministically(&mut decoder),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    /// Test empty map handling - special case mentioned in RFC 8949.
    /// An empty map is valid and must still follow length encoding rules
    /// from Section 4.2.1.
    #[test]
    fn test_empty_map() {
        let empty_map = vec![
            0xA0, // Map with 0 pairs - encoded with immediate value as per Section 4.2.1
        ];
        let mut decoder = Decoder::new(&empty_map);
        assert!(decode_map_deterministically(&mut decoder).is_ok());
    }

    /// Test minimal length encoding rules for maps as specified in RFC 8949 Section 4.2.1
    ///
    /// From RFC 8949 Section 4.2.1 "Integer Encoding":
    /// "The following must be encoded only with the shortest form that can represent
    /// the value:
    ///  1. Integer values in items that use integer encoding
    ///  2. The length of arrays, maps, strings, and byte strings
    ///
    /// Specifically for integers:
    ///  * 0 to 23 must be expressed in the same byte as the major type
    ///  * 24 to 255 must be expressed only with an additional uint8_t
    ///  * 256 to 65535 must be expressed only with an additional uint16_t
    ///  * 65536 to 4294967295 must be expressed only with an additional uint32_t"
    ///
    /// For maps (major type 5), the length must follow these rules. This ensures
    /// a deterministic encoding where the same length is always encoded the same way.
    /// Test minimal length encoding rules for maps as specified in RFC 8949 Section 4.2.1
    ///
    /// From RFC 8949 Section 4.2.1:
    /// "The length of arrays, maps, strings, and byte strings must be encoded in the
    /// smallest possible way. For maps (major type 5), lengths 0-23 must be encoded
    /// in the initial byte."
    #[test]
    fn test_map_minimal_length_encoding() {
        // Test case 1: Valid minimal encoding (length = 1)
        let valid_small = vec![
            0xA1, // Map, length 1 (major type 5 with immediate value 1)
            0x01, // Key: unsigned int 1
            0x02, // Value: unsigned int 2
        ];
        let mut decoder = Decoder::new(&valid_small);
        assert!(decode_map_deterministically(&mut decoder).is_ok());

        // Test case 2: Invalid non-minimal encoding (using additional info 24 for length 1)
        let invalid_small = vec![
            0xB8, // Map with additional info = 24 (0xa0 | 0x18)
            0x01, // Length encoded as uint8 = 1
            0x01, // Key: unsigned int 1
            0x02, // Value: unsigned int 2
        ];
        let mut decoder = Decoder::new(&invalid_small);
        let result = decode_map_deterministically(&mut decoder);

        // Debug info
        println!("Result: {:?}", result);
        println!("Initial byte: 0x{:02x}", invalid_small[0]);

        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    /// Test rejection of indefinite-length maps as required by RFC 8949 Section 4.2.2
    ///
    /// From RFC 8949 Section 4.2.2:
    /// "Indefinite-length items must be made definite-length items."
    ///
    /// The specification explicitly prohibits indefinite-length items in
    /// deterministic encoding to ensure consistent representation.
    #[test]
    fn test_map_indefinite_length() {
        let indefinite_map = vec![
            0xBF, // Start indefinite-length map (major type 5, additional info 31)
            0x41, 0x01, // Key 1: 1-byte string
            0x41, 0x02, // Value 1: 1-byte string
            0xFF, // Break (end of indefinite-length map)
        ];
        let mut decoder = Decoder::new(&indefinite_map);
        assert!(matches!(
            decode_map_deterministically(&mut decoder),
            Err(DeterministicError::IndefiniteLength)
        ));
    }

    /// Test handling of complex key structures while maintaining canonical ordering
    ///
    /// RFC 8949 Section 4.2.3 requires correct ordering regardless of key complexity:
    /// "The keys in every map must be sorted [...] Note that this rule allows maps
    /// to be deterministically ordered regardless of the specific data model of
    /// the key values."
    #[test]
    fn test_map_complex_keys() {
        // Test nested structures in keys while maintaining order
        // Following RFC 8949 Section 4.2.3 length-first rule
        let valid_complex = vec![
            0xA2, // Map with 2 pairs
            0x42, 0x01, 0x02, // Key 1: simple 2-byte string (shorter, so comes first)
            0x41, 0x01, // Value 1
            0x44, 0x01, 0x02, 0x03, 0x04, // Key 2: 4-byte string (longer, so comes second)
            0x41, 0x02, // Value 2
        ];
        let mut decoder = Decoder::new(&valid_complex);
        assert!(decode_map_deterministically(&mut decoder).is_ok());
    }

    /// Test edge cases for map encoding while maintaining compliance with RFC 8949
    ///
    /// These cases test boundary conditions that must still follow all rules from
    /// Section 4.2:
    /// - Minimal length encoding (4.2.1)
    /// - No indefinite lengths (4.2.2)
    /// - Canonical ordering (4.2.3)
    #[test]
    fn test_map_edge_cases() {
        // Single entry map - must still follow minimal length encoding rules
        let single_entry = vec![
            0xA1, // Map with 1 pair (using immediate value as per Section 4.2.1)
            0x41, 0x01, // Key: 1-byte string
            0x41, 0x02, // Value: 1-byte string
        ];
        let mut decoder = Decoder::new(&single_entry);
        assert!(decode_map_deterministically(&mut decoder).is_ok());

        // Map with zero-length string key - tests smallest possible key case
        // Still must follow sorting rules from Section 4.2.3
        let zero_length_key = vec![
            0xA1, // Map with 1 pair
            0x40, // Key: 0-byte string (smallest possible key length)
            0x41, 0x01, // Value: 1-byte string
        ];
        let mut decoder = Decoder::new(&zero_length_key);
        assert!(decode_map_deterministically(&mut decoder).is_ok());
    }

    /// Test duplicate key detection as required by RFC 8949 Section 4.2.3
    ///
    /// From RFC 8949 Section 4.2.3:
    /// "The keys in every map must be sorted [...] Note that this rule
    /// automatically implies that no two keys in a map can be equal (have
    /// the same length and the same value)."
    #[test]
    fn test_duplicate_keys() {
        let map_with_duplicates = vec![
            0xA2, // Map with 2 pairs
            0x41, 0x01, // Key 1: 1-byte string [0x01]
            0x41, 0x02, // Value 1
            0x41, 0x01, // Key 2: same as Key 1 (duplicate - invalid)
            0x41, 0x03, // Value 2
        ];
        let mut decoder = Decoder::new(&map_with_duplicates);
        assert!(matches!(
            decode_map_deterministically(&mut decoder),
            Err(DeterministicError::DuplicateMapKey)
        ));
    }

    #[test]
    fn test_decoder_modes() {
        // Create a CBOR map with sorted keys
        let mut data = Vec::new();
        let mut enc = Encoder::new(&mut data);

        // Create a sorted map with definite length
        enc.map(2).unwrap(); // Map with 2 pairs
        enc.str("a").unwrap(); // Key 1 (alphabetically first)
        enc.u32(1).unwrap(); // Value 1
        enc.str("b").unwrap(); // Key 2
        enc.u32(2).unwrap(); // Value 2

        // Test non-deterministic mode
        let non_det_decoder = CborDecoder::new(false);
        let mut dec = Decoder::new(&data);
        let result = non_det_decoder.decode(&mut dec).unwrap();
        // Should preserve original bytes
        assert_eq!(result, data);

        // Test deterministic mode with sorted input
        let det_decoder = CborDecoder::new(true);
        let mut dec = Decoder::new(&data);
        let result = det_decoder.decode(&mut dec).unwrap();
        // Should accept already sorted map
        assert_eq!(result, data);

        // Now test with unsorted keys
        let mut unsorted_data = Vec::new();
        let mut enc = Encoder::new(&mut unsorted_data);
        enc.map(2).unwrap();
        enc.str("b").unwrap(); // Key 1 (unsorted)
        enc.u32(2).unwrap(); // Value 1
        enc.str("a").unwrap(); // Key 2
        enc.u32(1).unwrap(); // Value 2

        // Non-deterministic mode should still work with unsorted
        let mut dec = Decoder::new(&unsorted_data);
        let result = non_det_decoder.decode(&mut dec).unwrap();
        assert_eq!(result, unsorted_data);

        // Deterministic mode should reject unsorted keys
        let mut dec = Decoder::new(&unsorted_data);
        let result = det_decoder.decode(&mut dec);
        assert!(matches!(result, Err(DeterministicError::UnorderedMapKeys)));
    }
}
