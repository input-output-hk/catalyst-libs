//! CBOR map (CBOR major type 5) structure with CBOR decoding and encoding functionality.
//! Supports deterministically encoded rules (RFC 8949 Section 4.2) if corresponding
//! option is enabled.

use std::{cmp::Ordering, vec::IntoIter};

use crate::{
    decode_helper::get_bytes,
    deterministic_helper::{get_cbor_header_size, get_declared_length, CBOR_MAX_TINY_VALUE},
};

/// Represents a CBOR map key-value pair, preserving original decoding order of values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map(pub Vec<MapEntry>);

impl IntoIterator for Map {
    type IntoIter = IntoIter<MapEntry>;
    type Item = MapEntry;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Represents a CBOR map key-value pair where the key must be deterministically encoded
/// according to RFC 8949 Section 4.2.3.
///
/// This type stores the raw bytes of both key and value to enable:
/// 1. Length-first ordering of keys (shorter keys before longer ones)
/// 2. Lexicographic comparison of equal-length keys
/// 3. Preservation of the original encoded form
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MapEntry {
    /// Raw bytes of the encoded key, used for deterministic ordering
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

/// Major type indicator for CBOR maps (major type 5: 101 in top 3 bits)
/// As per RFC 8949 Section 4.2.3, maps in deterministic encoding must:
/// - Have keys sorted by length first, then byte wise lexicographically
/// - Contain no duplicate keys
const CBOR_MAJOR_TYPE_MAP: u8 = 5 << 5;

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
impl minicbor::Decode<'_, ()> for Map {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // From RFC 8949 Section 4.2.2:
        // "Indefinite-length items must be made definite-length items."
        // The specification explicitly prohibits indefinite-length items in
        // deterministic encoding to ensure consistent representation.
        let map_len = d.map()?.ok_or_else(|| {
            minicbor::decode::Error::message(
                "Indefinite-length items must be made definite-length items",
            )
        })?;

        let header_end_pos = d.position();
        check_map_minimal_length(d, header_end_pos, map_len)?;

        // Decode entries to validate them
        let entries = decode_map_entries(d, map_len)?;

        validate_map_ordering(&entries)?;

        Ok(Self(entries))
    }
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
    decoder: &minicbor::Decoder, position: usize, value: u64,
) -> Result<(), minicbor::decode::Error> {
    // For zero length, 0xA0 is always the minimal encoding
    if value == 0 {
        return Ok(());
    }

    let initial_byte = decoder.input().get(position).copied().ok_or_else(|| {
        minicbor::decode::Error::message(minicbor::decode::Error::message(
            "Cannot read initial byte for minimality check",
        ))
    })?;
    // Only check minimality for map length encodings using uint8
    // Immediate values (0-23) are already minimal by definition
    if initial_byte == CBOR_MAP_LENGTH_UINT8 && value <= CBOR_MAX_TINY_VALUE {
        return Err(minicbor::decode::Error::message(
            minicbor::decode::Error::message("map minimal length failure"),
        ));
    }

    Ok(())
}

/// Decodes all key-value pairs in the map
fn decode_map_entries(
    d: &mut minicbor::Decoder, length: u64,
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
        map_keys_are_deterministic(&key_bytes)?;

        let value = get_bytes(d, value_start, value_end)?;

        entries.push(MapEntry { key_bytes, value });
    }

    Ok(entries)
}

/// Validates that a CBOR map key follows the deterministic encoding rules as specified in
/// RFC 8949. In this case, it validates that the keys themselves must be
/// deterministically encoded (4.2.1).
fn map_keys_are_deterministic(key_bytes: &[u8]) -> Result<(), minicbor::decode::Error> {
    // if the map keys are not a txt string or byte string we cannot get a declared length
    if let Some(key_declared_length) = get_declared_length(key_bytes)? {
        let header_size = get_cbor_header_size(key_bytes)?;
        let actual_content_size = key_bytes.len().checked_sub(header_size).ok_or_else(|| {
            minicbor::decode::Error::message("Integer overflow in content size calculation")
        })?;

        if key_declared_length != actual_content_size {
            return Err(minicbor::decode::Error::message(
                "Declared length does not match the actual length. Non deterministic map key.",
            ));
        }
    }
    Ok(())
}

/// Validates map keys are properly ordered according to RFC 8949 Section 4.2.3
/// and checks for duplicate keys
fn validate_map_ordering(entries: &[MapEntry]) -> Result<(), minicbor::decode::Error> {
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
fn check_pair_ordering(current: &MapEntry, next: &MapEntry) -> Result<(), minicbor::decode::Error> {
    match current.cmp(next) {
        Ordering::Less => Ok(()), // Valid: keys are in ascending order
        Ordering::Equal => Err(minicbor::decode::Error::message("Duplicate map key found")),
        Ordering::Greater => {
            Err(minicbor::decode::Error::message(
                "Map keys not in canonical order",
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use minicbor::{Decode, Decoder};

    use super::*;

    /// Ensures that encoding and decoding a map preserves:
    /// - The byte wise lexicographic ordering of keys
    /// - The exact byte representation of values
    /// - The definite length encoding format
    #[test]
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
        let result = Map::decode(&mut decoder, &mut ()).unwrap();

        // Verify we got back exactly the same bytes

        assert_eq!(
            result,
            Map(vec![
                MapEntry {
                    // Key 1: 2-byte string
                    key_bytes: vec![0x42, 0x01, 0x02],
                    // Value 1: 1-byte string
                    value: vec![0x41, 0x01]
                },
                MapEntry {
                    // Key 2: 3-byte string
                    key_bytes: vec![0x43, 0x01, 0x02, 0x03,],
                    // Value 2: 1-byte string
                    value: vec![0x41, 0x02,]
                }
            ])
        );
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
        assert!(Map::decode(&mut decoder, &mut ()).is_ok());

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
        assert!(Map::decode(&mut decoder, &mut ()).is_err());
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
        assert!(Map::decode(&mut decoder, &mut ()).is_ok());
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

        assert!(Map::decode(&mut decoder, &mut ()).is_ok());

        // Test case 2: Invalid non-minimal encoding (using additional info 24 for length 1)
        let invalid_small = vec![
            0xB8, // Map with additional info = 24 (0xa0 | 0x18)
            0x01, // Length encoded as uint8 = 1
            0x01, // Key: unsigned int 1
            0x02, // Value: unsigned int 2
        ];
        let mut decoder = Decoder::new(&invalid_small);
        assert!(Map::decode(&mut decoder, &mut ()).is_err());
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
        assert!(Map::decode(&mut decoder, &mut ()).is_ok());
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
        assert!(Map::decode(&mut decoder, &mut ()).is_ok());

        // Map with zero-length string key - tests smallest possible key case
        // Still must follow sorting rules from Section 4.2.3
        let zero_length_key = vec![
            0xA1, // Map with 1 pair
            0x40, // Key: 0-byte string (smallest possible key length)
            0x41, 0x01, // Value: 1-byte string
        ];
        let mut decoder = Decoder::new(&zero_length_key);
        assert!(Map::decode(&mut decoder, &mut ()).is_ok());
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
        assert!(Map::decode(&mut decoder, &mut ()).is_err());
    }

    #[test]
    fn test_map_entry_ord_comprehensive() {
        // Test 1: Length-first ordering
        // According to RFC 8949, shorter keys must come before longer keys
        // regardless of their actual byte values
        let short_key = MapEntry {
            key_bytes: vec![0x41], // Single byte key
            value: vec![0x01],
        };
        let long_key = MapEntry {
            key_bytes: vec![0x41, 0x42, 0x43], // Three byte key (longer)
            value: vec![0x01],
        };
        // Even though both start with 0x41, the shorter one comes first
        assert!(short_key < long_key);
        assert!(long_key > short_key);

        // Test 2: Lexicographic ordering for equal-length keys
        // When keys have the same length, they are compared byte by byte
        // lexicographically (like dictionary ordering)
        let key_a = MapEntry {
            key_bytes: vec![0x41, 0x41], // Represents "AA" in ASCII
            value: vec![0x01],
        };
        let key_b = MapEntry {
            key_bytes: vec![0x41, 0x42], // Represents "AB" in ASCII
            value: vec![0x01],
        };
        // "AA" comes before "AB" lexicographically
        assert!(key_a < key_b);
        assert!(key_b > key_a);
        assert!(key_a == key_a);

        // Test 3: Identical entries (same key AND value)
        // Complete MapEntry equality requires both key and value to be identical
        let entry1 = MapEntry {
            key_bytes: vec![0x41, 0x42],
            value: vec![0x01],
        };
        let entry2 = MapEntry {
            key_bytes: vec![0x41, 0x42],
            value: vec![0x01], // Same value as entry1
        };
        // These are truly identical entries
        assert_eq!(entry1, entry2);

        // Test 4: Same key, different values - these are NOT equal
        // In CBOR maps, this would represent duplicate keys (invalid)
        let entry_v1 = MapEntry {
            key_bytes: vec![0x41, 0x42],
            value: vec![0x01],
        };
        let entry_v2 = MapEntry {
            key_bytes: vec![0x41, 0x42],
            value: vec![0x02], // Different value
        };
        // These entries are NOT equal (different values)
        assert_ne!(entry_v1, entry_v2);
        // But they have the same ordering position (same key)
        assert_eq!(entry_v1.cmp(&entry_v2), std::cmp::Ordering::Equal);

        // Test 5: Empty key vs non-empty key
        // Empty keys should come before any non-empty key (shortest length rule)
        let empty_key = MapEntry {
            key_bytes: vec![], // Empty key (length 0)
            value: vec![0x01],
        };
        let non_empty_key = MapEntry {
            key_bytes: vec![0x00], // Single null byte (length 1)
            value: vec![0x01],
        };
        // Empty key (length 0) comes before single byte key (length 1)
        assert!(empty_key < non_empty_key);

        // Test 6: Numerical byte value ordering
        // Test that individual byte values are compared correctly (0x00 < 0xFF)
        let key_0 = MapEntry {
            key_bytes: vec![0x00], // Null byte
            value: vec![0x01],
        };
        let key_255 = MapEntry {
            key_bytes: vec![0xFF], // Maximum byte value
            value: vec![0x01],
        };
        // 0x00 is numerically less than 0xFF
        assert!(key_0 < key_255);

        // Test 7: Complex multi-byte lexicographic comparison
        // Test lexicographic ordering when keys differ in later bytes
        let key_complex1 = MapEntry {
            key_bytes: vec![0x01, 0x02, 0x03], // Differs in last byte (0x03)
            value: vec![0x01],
        };
        let key_complex2 = MapEntry {
            key_bytes: vec![0x01, 0x02, 0x04], // Differs in last byte (0x04)
            value: vec![0x01],
        };
        // First two bytes are identical (0x01, 0x02), so compare third byte: 0x03 < 0x04
        assert!(key_complex1 < key_complex2);
    }
    /// An edge case where slice [`Ord`] isn't minimal length byte-wise lexicographic.
    #[test]
    fn test_map_entry_ord_len_edge_case() {
        // Shorter length key with greater first byte.
        let lhs = MapEntry {
            key_bytes: minicbor::to_vec("a").unwrap(),
            value: vec![],
        };
        assert_eq!(lhs.key_bytes, &[97, 97]);

        // Longer length key with lesser first byte.
        let rhs = MapEntry {
            key_bytes: minicbor::to_vec(65535u32).unwrap(),
            value: vec![],
        };
        assert_eq!(rhs.key_bytes, &[25, 255, 255]);

        // Shorter must go first.
        assert!(lhs < rhs);
    }
}
