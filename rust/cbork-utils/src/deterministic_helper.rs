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

/// CBOR Major Type for Maps (5) shifted left by 5 bits
const CBOR_MAP_TYPE: u8 = 5 << 5;

/// CBOR header byte for indefinite-length maps (major type 5, additional info 31)
const CBOR_INDEFINITE_MAP: u8 = CBOR_MAP_TYPE | 31;

/// CBOR Major Type for Arrays (4) shifted left by 5 bits
const CBOR_ARRAY_TYPE: u8 = 4 << 5;

/// CBOR array headers for different length encodings
const CBOR_ARRAY_UINT8: u8 = CBOR_ARRAY_TYPE | 24; // 0x98
const CBOR_ARRAY_UINT16: u8 = CBOR_ARRAY_TYPE | 25; // 0x99
const CBOR_ARRAY_UINT32: u8 = CBOR_ARRAY_TYPE | 26; // 0x9A
const CBOR_ARRAY_UINT64: u8 = CBOR_ARRAY_TYPE | 27; // 0x9B

/// CBOR Major Type for Text Strings (3) shifted left by 5 bits
const CBOR_TEXT_STRING_TYPE: u8 = 3 << 5;

/// CBOR Major Type for Byte Strings (2) shifted left by 5 bits
const CBOR_BYTE_STRING_TYPE: u8 = 2 << 5;

/// CBOR string length encodings
const CBOR_STRING_UINT8: u8 = 24;
const CBOR_STRING_UINT16: u8 = 25;
const CBOR_STRING_UINT32: u8 = 26;
const CBOR_STRING_UINT64: u8 = 27;

/// CBOR headers for indefinite-length strings
const CBOR_INDEFINITE_TEXT: u8 = CBOR_TEXT_STRING_TYPE | 31;
const CBOR_INDEFINITE_BYTES: u8 = CBOR_BYTE_STRING_TYPE | 31;

/// Maximum values for each compact representation
const MAX_VALUE_UINT8: u64 = 23;
const MAX_VALUE_UINT16: u64 = u8::MAX as u64;
const MAX_VALUE_UINT32: u64 = u16::MAX as u64;
const MAX_VALUE_UINT64: u64 = u32::MAX as u64;

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
pub fn decode_map_deterministically(d: &mut Decoder) -> Result<Vec<MapEntry>, DeterministicError> {
    validate_input_not_empty(d)?;
    validate_not_indefinite_length_map(d)?;

    let start_pos = d.position();
    let map_len = d.map()?.ok_or(DeterministicError::UnexpectedEof)?;

    check_minimal_length(d, start_pos, map_len)?;
    let entries = decode_map_entries(d, map_len)?;
    validate_map_ordering(&entries)?;

    Ok(entries)
}

/// Validates that map does not use indefinite-length encoding
fn validate_not_indefinite_length_map(d: &Decoder) -> Result<(), DeterministicError> {
    let initial_byte = d.input()[d.position()];
    if initial_byte == CBOR_INDEFINITE_MAP {
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
    major_type == CBOR_TEXT_STRING_TYPE || major_type == CBOR_BYTE_STRING_TYPE
}

/// Checks if the byte represents an indefinite-length string
#[inline]
fn is_indefinite_string(byte: u8) -> bool {
    byte == CBOR_INDEFINITE_TEXT || byte == CBOR_INDEFINITE_BYTES
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

        CBOR_STRING_UINT8 => {
            let bytes = get_checked_slice(input, start_pos + 1, 1)?;
            Ok(u64::from(bytes[0]))
        },

        CBOR_STRING_UINT16 => {
            let bytes = get_checked_slice(input, start_pos + 1, 2)?;
            Ok(u64::from(u16::from_be_bytes(bytes.try_into().unwrap())))
        },

        CBOR_STRING_UINT32 => {
            let bytes = get_checked_slice(input, start_pos + 1, 4)?;
            Ok(u64::from(u32::from_be_bytes(bytes.try_into().unwrap())))
        },

        CBOR_STRING_UINT64 => {
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
        CBOR_STRING_UINT8 => {
            if length <= 23 {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_UINT16 => {
            if u8::try_from(length).is_ok() {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_UINT32 => {
            if u16::try_from(length).is_ok() {
                return Err(DeterministicError::NonMinimalInt);
            }
        },
        CBOR_STRING_UINT64 => {
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
    use std::cmp::Ordering;

    use super::*;

    /// Test the map key comparison logic used for deterministic ordering
    /// as specified in RFC 8949 Section 4.2.3.
    ///
    /// The RFC states two rules for key ordering:
    /// 1. "If two keys have different lengths, the shorter one sorts earlier;"
    /// 2. "If two keys have the same length, the one with the lower value in (byte-wise)
    ///    lexical order sorts earlier."
    #[test]
    fn test_map_key_ordering() {
        // Test case 1: Keys of different lengths
        // RFC 8949 4.2.3 Rule 1: "If two keys have different lengths, the shorter one sorts
        // earlier"
        let shorter_key = MapEntry {
            key_bytes: vec![1, 2],
            value: vec![],
        };
        let longer_key = MapEntry {
            key_bytes: vec![1, 2, 3],
            value: vec![],
        };
        assert_eq!(shorter_key.compare(&longer_key), Ordering::Less);

        // Test case 2: Equal length keys, different values
        // RFC 8949 4.2.3 Rule 2: "If two keys have the same length, the one with the lower value
        // in (byte-wise) lexical order sorts earlier"
        let key1 = MapEntry {
            key_bytes: vec![1, 2, 3],
            value: vec![],
        };
        let key2 = MapEntry {
            key_bytes: vec![1, 2, 4],
            value: vec![],
        };
        assert_eq!(key1.compare(&key2), Ordering::Less);

        // Test case 3: Equal keys
        // RFC 8949 4.2.3: "No two keys in a map may be equal"
        // This case should never occur in valid CBOR but we test the comparison behavior
        let key3 = MapEntry {
            key_bytes: vec![1, 2, 3],
            value: vec![],
        };
        let key4 = MapEntry {
            key_bytes: vec![1, 2, 3],
            value: vec![],
        };
        assert_eq!(key3.compare(&key4), Ordering::Equal);
    }

    /// Test the deterministic map validation rules from RFC 8949 Section 4.2.3.
    ///
    /// The RFC mandates:
    /// 1. Keys must be sorted by length first
    /// 2. Equal length keys must be sorted lexicographically
    /// 3. No duplicate keys are allowed
    ///
    /// Section 4.2.3: "The keys in every map must be sorted in the following order:
    /// 1. If two keys have different lengths, the shorter one sorts earlier;
    /// 2. If two keys have the same length, the one with the lower value in (byte-wise)
    ///    lexical order sorts earlier."
    #[test]
    fn test_map_validation() {
        // Test case 1: Valid ordering - shorter key before longer key
        // RFC 8949 4.2.3 Example: a one-byte key must sort before a two-byte key
        let valid_entries = vec![
            MapEntry {
                key_bytes: vec![1, 2], // Length 2 key
                value: vec![],
            },
            MapEntry {
                key_bytes: vec![1, 2, 3], // Length 3 key
                value: vec![],
            },
        ];
        assert!(validate_map_ordering(&valid_entries).is_ok());

        // Test case 2: Invalid ordering - longer key before shorter key
        // RFC 8949 4.2.3: Violates rule "shorter one sorts earlier"
        let invalid_entries = vec![
            MapEntry {
                key_bytes: vec![1, 2, 3], // Length 3 key
                value: vec![],
            },
            MapEntry {
                key_bytes: vec![1, 2], // Length 2 key
                value: vec![],
            },
        ];
        assert!(matches!(
            validate_map_ordering(&invalid_entries),
            Err(DeterministicError::UnorderedMapKeys)
        ));

        // Test case 3: Duplicate keys
        // RFC 8949 4.2.3: "No two keys in a map may be equal"
        let duplicate_entries = vec![
            MapEntry {
                key_bytes: vec![1, 2],
                value: vec![],
            },
            MapEntry {
                key_bytes: vec![1, 2], // Same key bytes as above
                value: vec![],
            },
        ];
        assert!(matches!(
            validate_map_ordering(&duplicate_entries),
            Err(DeterministicError::DuplicateMapKey)
        ));
    }

    /// Test string length encoding validation according to RFC 8949 Section 4.2.1 and
    /// 4.2.2.
    ///
    /// Section 4.2.1 mandates minimal encoding for lengths:
    /// - 0 to 23: must be expressed in the same byte as the major type
    /// - 24 to 255: must use additional uint8
    /// - 256 to 65535: must use additional uint16
    /// - 65536 to 4294967295: must use additional uint32
    /// - 4294967296 to 18446744073709551615: must use additional uint64
    ///
    /// Section 4.2.2: "Indefinite-length items must be made definite"
    #[test]
    fn test_string_length_validation() {
        // Test case 1: Valid minimal encoding for small string
        // RFC 8949 4.2.1: Length 3 should be encoded in the same byte as major type
        let valid_small = vec![
            0x63, // Text string, length 3 (0x60 | 3)
            b'f', b'o', b'o',
        ];
        let decoder = Decoder::new(&valid_small);
        assert!(validate_string_length(&decoder, 0).is_ok());

        // Test case 2: Non-minimal encoding for small string
        // RFC 8949 4.2.1: "The value 24 MUST NOT be used if the value can be encoded in fewer
        // bytes"
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
        // RFC 8949 4.2.1: Length 128 must use uint8 encoding as it's > 23
        let mut valid_medium = vec![
            0x78, 0x80, // Text string, length 128 with uint8
        ];
        valid_medium.extend(vec![b'x'; 128]);
        let decoder = Decoder::new(&valid_medium);
        assert!(validate_string_length(&decoder, 0).is_ok());

        // Test case 4: Indefinite length string
        // RFC 8949 4.2.2: "Indefinite-length items must be made definite"
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
}
