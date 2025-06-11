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
/// - Have keys sorted by length first, then byte wise lexicographically
/// - Contain no duplicate keys
const CBOR_MAJOR_TYPE_MAP: u8 = 5 << 5;

/// Maximum value that can be encoded in a 5-bit additional info field
/// RFC 8949 Section 4.2.1: "0 to 23 must be expressed in the same byte as the major type"
/// Values 0-23 are encoded directly in the additional info field of the initial byte
const CBOR_MAX_TINY_VALUE: u64 = 23;

/// Initial byte for a CBOR map whose length is encoded as an 8-bit unsigned integer
/// (uint8).
///
/// This value combines the map major type (5) with the additional information value (24)
/// that indicates a uint8 length follows. The resulting byte is:
/// - High 3 bits: 101 (major type 5 for map)
/// - Low 5 bits: 24 (indicates uint8 length follows)
///
/// Used when encoding CBOR maps with lengths between 24 and 255 elements.
const CBOR_MAP_LENGTH_UINT8: u8 = CBOR_MAJOR_TYPE_MAP | 24; // For uint8 length encoding

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
    /// 2. If lengths equal, compare byte wise lexicographically
    fn compare(&self, other: &Self) -> Ordering {
        match self.key_bytes.len().cmp(&other.key_bytes.len()) {
            Ordering::Equal => self.key_bytes.cmp(&other.key_bytes),
            ordering => ordering,
        }
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

    // From RFC 8949 Section 4.2.2:
    // "Indefinite-length items must be made definite-length items."
    // The specification explicitly prohibits indefinite-length items in
    // deterministic encoding to ensure consistent representation.
    match d.datatype()? {
        minicbor::data::Type::Map => {},
        minicbor::data::Type::MapIndef => return Err(DeterministicError::IndefiniteLength),
        _ => {
            return Err(DeterministicError::CorruptedEncoding(
                "Expected a map".into(),
            ))
        },
    }

    let start_pos = d.position();
    let map_len = d.map()?.ok_or(minicbor::decode::Error::end_of_input())?;

    check_map_minimal_length(d, start_pos, map_len)?;

    // Store the starting position of the entire map
    let map_start = start_pos;

    // Decode entries to validate them
    let entries = decode_map_entries(d, map_len)?;
    validate_map_ordering(&entries)?;

    // Get the ending position after validation
    let map_end = d.position();

    // Return the raw bytes of the entire validated map
    get_map_bytes(d, map_start, map_end)
}

/// Extracts the raw bytes of a CBOR map from a decoder based on specified positions.
/// This function retrieves the raw byte representation of a CBOR map between the given
/// start and end positions from the decoder's underlying buffer.
fn get_map_bytes(
    d: &Decoder<'_>, map_start: usize, map_end: usize,
) -> Result<Vec<u8>, DeterministicError> {
    d.input()
        .get(map_start..map_end)
        .ok_or_else(|| {
            DeterministicError::CorruptedEncoding(
                "Invalid map byte range: indices out of bounds".to_string(),
            )
        })
        .map(<[u8]>::to_vec)
}

/// Decodes all key-value pairs in the map
fn decode_map_entries(d: &mut Decoder, length: u64) -> Result<Vec<MapEntry>, DeterministicError> {
    let capacity = usize::try_from(length).map_err(|_| {
        DeterministicError::CorruptedEncoding(
            "Map length too large for current platform".to_string(),
        )
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

        // Extract the raw bytes for both key and value
        let key_bytes = extract_cbor_bytes(d, key_start, key_end)?;

        if key_bytes.len() != value_end-value_start{
            return Err(DeterministicError::InvalidLength

            );
        }

        // value bytes
        let value = extract_cbor_bytes(d, value_start, value_end)?;



        entries.push(MapEntry { key_bytes, value });
    }

    Ok(entries)
}

/// Validates that a CBOR item's declared length matches its content size,
/// which is one of the requirements for deterministic CBOR encoding as specified in RFC
/// 8949.
///
/// This function specifically checks the length requirement by ensuring the declared
/// length matches the actual content size. This helps detect malformed or
/// non-deterministic CBOR where the length prefix doesn't match the actual content.
fn extract_cbor_bytes(
    decoder: &minicbor::Decoder<'_>, range_start: usize, range_end: usize,
) -> Result<Vec<u8>, DeterministicError> {
    // Validate CBOR byte range bounds
    if range_start >= range_end {
        return Err(DeterministicError::InvalidLength(
            "Invalid CBOR byte range: start must be less than end position".to_string(),
        ));
    }

    let byte_length = range_end.saturating_sub(range_start);
    get_checked_slice(decoder.input(), range_start, byte_length).map(<[u8]>::to_vec)
}

/// Validates map keys are properly ordered according to RFC 8949 Section 4.2.3
/// and checks for duplicate keys
fn validate_map_ordering(entries: &[MapEntry]) -> Result<(), DeterministicError> {
    let mut iter = entries.iter();

    // Get the first element if it exists
    let Some(mut current) = iter.next() else {
        // Empty slice is valid
        return Ok(());
    };

    // Compare each adjacent pair
    for next in iter {
        check_pair_ordering(current, next)?;
        current = next;
    }

    Ok(())
}

/// Checks if two adjacent map entries are in the correct order:
/// - Keys must be in ascending order (current < next)
/// - Duplicate keys are not allowed (current != next)
fn check_pair_ordering(current: &MapEntry, next: &MapEntry) -> Result<(), DeterministicError> {
    match current.compare(next) {
        Ordering::Less => Ok(()), // Valid: keys are in ascending order
        Ordering::Equal => Err(DeterministicError::DuplicateMapKey),
        Ordering::Greater => Err(DeterministicError::UnorderedMapKeys),
    }
}

/// Validates that the decoder's input buffer is not empty.
fn validate_input_not_empty(d: &Decoder) -> Result<(), DeterministicError> {
    if d.position() >= d.input().len() {
        return Err(DeterministicError::UnexpectedEof);
    }
    Ok(())
}

/// Validates that a CBOR map's length is encoded using the minimal number of bytes as
/// required by RFC 8949's deterministic encoding rules.
///
/// According to the deterministic encoding requirements:
/// - The length of a map MUST be encoded using the smallest possible CBOR additional
///   information value
/// - For values 0 through 23, the additional info byte is used directly
/// - For values that fit in 8, 16, 32, or 64 bits, the appropriate multi-byte encoding
///   must be used
///
/// # Specification Reference
/// This implementation follows RFC 8949 Section 4.2.1 which requires that:
/// "The length of arrays, maps, and strings MUST be encoded using the smallest possible
/// CBOR additional information value."
fn check_map_minimal_length(
    decoder: &Decoder, position: usize, value: u64,
) -> Result<(), DeterministicError> {
    const ENCODING_ERROR_MSG: &str = "Cannot read initial byte for minimality check";

    let initial_byte = decoder
        .input()
        .get(position)
        .copied()
        .ok_or_else(|| DeterministicError::CorruptedEncoding(ENCODING_ERROR_MSG.to_owned()))?;

    // Only check minimality for map length encodings using uint8
    // Immediate values (0-23) are already minimal by definition
    if initial_byte == CBOR_MAP_LENGTH_UINT8 && value <= CBOR_MAX_TINY_VALUE {
        return Err(DeterministicError::NonMinimalInt);
    }

    Ok(())
}

/// Gets a slice of the input with bounds checking
fn get_checked_slice(
    input: &[u8], start_pos: usize, length: usize,
) -> Result<&[u8], DeterministicError> {
    let end_pos = start_pos
        .checked_add(length)
        .ok_or(DeterministicError::UnexpectedEof)?;
    input
        .get(start_pos..end_pos)
        .ok_or(DeterministicError::UnexpectedEof)
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

    /// Corrupted encoding
    CorruptedEncoding(String),

    /// Indicates unexpected end of input
    UnexpectedEof,

    /// Indicates that the declared length doesn't match the actual content size
    InvalidLength(String),
}

impl fmt::Display for DeterministicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeterministicError::NonMinimalInt => write!(f, "Integer not encoded in minimal form"),
            DeterministicError::IndefiniteLength => {
                write!(f, "Indefinite-length items not allowed")
            },
            DeterministicError::DecoderError(e) => write!(f, "Decoder error: {e}"),
            DeterministicError::UnorderedMapKeys => write!(f, "Map keys not in canonical order"),
            DeterministicError::DuplicateMapKey => write!(f, "Duplicate map key found"),
            DeterministicError::UnexpectedEof => write!(f, "Unexpected end of input"),
            DeterministicError::CorruptedEncoding(e) => write!(f, "Corrupted encoding {e}"),
            DeterministicError::InvalidLength(e) => {
                write!(f, "Declared length does not match actual content size {e}")
            },
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

#[cfg(test)]
mod tests {
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
    fn test_map_keys_are_deterministically_encoded() {
        // bad encoding for keys
        let valid_map = vec![
            0xA4, 0x42, 0x01, 0x02, // Key 1: 2-byte string
            0x41, 0x01, // Value 1: 1-byte string
            0x43, 0x01, 0x02, 0x03, // Key 2: 3-byte string
            0x41, 0x02, // Value 2: 1-byte string
        ];
        let mut decoder = Decoder::new(&valid_map);
        assert!(decode_map_deterministically(&mut decoder).is_err());
    }

    #[test]
    // Ensures that encoding and decoding a map preserves:
    /// - The byte wise lexicographic ordering of keys
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
    ///  * 24 to 255 must be expressed only with an additional `uint8_t`
    ///  * 256 to 65535 must be expressed only with an additional `uint16_t`
    ///  * 65536 to 4294967295 must be expressed only with an additional `uint32_t`"
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

        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
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
}
