//! CBOR array (CBOR major type 4) structure with CBOR decoding and encoding
//! functionality. Supports deterministically encoded rules (RFC 8949 Section 4.2) if
//! corresponding option is enabled.

use std::{ops::Deref, vec::IntoIter};

use crate::{
    decode_context::DecodeCtx,
    decode_helper::get_bytes,
    deterministic_helper::{get_cbor_header_size, get_declared_length, CBOR_MAX_TINY_VALUE},
};

/// Represents a CBOR array, preserving original decoding order of values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Array(pub Vec<Vec<u8>>);

impl Deref for Array {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Array {
    type IntoIter = IntoIter<Vec<u8>>;
    type Item = Vec<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Major type indicator for CBOR arrays (major type 4: 100 in top 3 bits)
/// As per RFC 8949 Section 4.2, arrays in deterministic encoding must:
/// - Have lengths encoded minimally (Section 4.2.1)
/// - Use definite-length encoding only (Section 4.2.2)
/// - Have all elements themselves deterministically encoded
const CBOR_MAJOR_TYPE_ARRAY: u8 = 4 << 5;

/// Initial byte for a CBOR array whose length is encoded as an 8-bit unsigned integer
/// (uint8).
///
/// This value combines the array major type (4) with the additional information value
/// (24) that indicates a uint8 length follows. The resulting byte is:
/// - High 3 bits: 100 (major type 4 for array)
/// - Low 5 bits: 24 (indicates uint8 length follows)
///
/// Used when encoding CBOR arrays with lengths between 24 and 255 elements.
const CBOR_ARRAY_LENGTH_UINT8: u8 = CBOR_MAJOR_TYPE_ARRAY | 24; // For uint8 length encoding

/// Decodes a CBOR array with deterministic encoding validation (RFC 8949 Section 4.2)
/// Returns the raw bytes of the array elements if it passes all deterministic validation
/// rules.
///
/// From RFC 8949 Section 4.2:
/// Arrays must follow these deterministic encoding rules:
/// - Array lengths must use minimal encoding (Section 4.2.1)
/// - Indefinite-length arrays are not allowed (Section 4.2.2)
/// - All array elements must themselves be deterministically encoded
///
/// # Errors
///
/// Returns `DeterministicError` if:
/// - Input is empty (`UnexpectedEof`)
/// - Array uses indefinite-length encoding (`IndefiniteLength`)
/// - Array length is not encoded minimally (`NonMinimalInt`)
/// - Array element decoding fails (`DecoderError`)
/// - Array elements are not deterministically encoded
impl minicbor::Decode<'_, DecodeCtx> for Array {
    fn decode(
        d: &mut minicbor::Decoder<'_>, ctx: &mut DecodeCtx,
    ) -> Result<Self, minicbor::decode::Error> {
        // Capture position before reading the array header
        let header_start_pos = d.position();

        // Handle both definite and indefinite-length arrays
        let array_len = d.array()?;

        match array_len {
            Some(length) => {
                // Definite-length array
                if matches!(ctx, DecodeCtx::Deterministic) {
                    ctx.try_check(|| check_array_minimal_length(d, header_start_pos, length))?;
                }

                let elements = decode_array_elements(d, length, ctx)?;
                Ok(Self(elements))
            },
            None => {
                // Indefinite-length array
                if matches!(ctx, DecodeCtx::Deterministic) {
                    return Err(minicbor::decode::Error::message(
                        "Indefinite-length items must be made definite-length items",
                    ));
                }

                // In non-deterministic mode, accept indefinite-length arrays
                // minicbor should handle the indefinite-length decoding for us
                // We'll use Vec<minicbor::data::Type> to decode heterogeneous elements
                let mut elements = Vec::new();

                // Since we can't easily determine when indefinite arrays end,
                // we'll need to work with the raw bytes approach
                let remaining_input = &d.input()[d.position()..];
                let mut temp_decoder = minicbor::Decoder::new(remaining_input);

                // Decode elements until we hit the break marker (0xFF)
                while temp_decoder.position() < temp_decoder.input().len() {
                    // Check if we've hit the break marker
                    if temp_decoder.input().get(temp_decoder.position()) == Some(&0xFF) {
                        // Skip the break marker
                        temp_decoder.skip().ok();
                        break;
                    }

                    let element_start = temp_decoder.position();
                    if temp_decoder.skip().is_err() {
                        break;
                    }
                    let element_end = temp_decoder.position();

                    if element_end > element_start {
                        let element_bytes =
                            temp_decoder.input()[element_start..element_end].to_vec();
                        elements.push(element_bytes);
                    }
                }

                // Update the main decoder position
                d.set_position(d.position() + temp_decoder.position());

                Ok(Self(elements))
            },
        }
    }
}

/// Validates that a CBOR array's length is encoded using the minimal number of bytes as
/// required by RFC 8949's deterministic encoding rules.
///
/// According to the deterministic encoding requirements:
/// - The length of an array MUST be encoded using the smallest possible CBOR additional
///   information value
/// - For values 0 through 23, the additional info byte is used directly
/// - For values that fit in 8, 16, 32, or 64 bits, the appropriate multi-byte encoding
///   must be used
///
/// # Specification Reference
/// This implementation follows RFC 8949 Section 4.2.1 which requires that:
/// "The length of arrays, maps, and strings MUST be encoded using the smallest possible
/// CBOR additional information value."
fn check_array_minimal_length(
    decoder: &minicbor::Decoder, header_start_pos: usize, value: u64,
) -> Result<(), minicbor::decode::Error> {
    // For zero length, 0x80 is always the minimal encoding
    if value == 0 {
        return Ok(());
    }

    let initial_byte = decoder
        .input()
        .get(header_start_pos)
        .copied()
        .ok_or_else(|| {
            minicbor::decode::Error::message("Cannot read initial byte for minimality check")
        })?;

    // Only check minimality for array length encodings using uint8
    // Immediate values (0-23) are already minimal by definition
    if initial_byte == CBOR_ARRAY_LENGTH_UINT8 && value <= CBOR_MAX_TINY_VALUE {
        return Err(minicbor::decode::Error::message(
            "array minimal length failure",
        ));
    }

    Ok(())
}

/// Decodes all elements in the array
fn decode_array_elements(
    d: &mut minicbor::Decoder, length: u64, ctx: &mut DecodeCtx,
) -> Result<Vec<Vec<u8>>, minicbor::decode::Error> {
    let capacity = usize::try_from(length).map_err(|_| {
        minicbor::decode::Error::message("Array length too large for current platform")
    })?;
    let mut elements = Vec::with_capacity(capacity);

    // Decode each array element
    for _ in 0..length {
        // Record the starting position of the element
        let element_start = d.position();

        // Skip over the element to find its end position
        d.skip()?;
        let element_end = d.position();

        // The elements themselves must be deterministically encoded (4.2.1)
        let element_bytes = get_bytes(d, element_start, element_end)?.to_vec();

        // Only check deterministic encoding in deterministic mode
        if matches!(ctx, DecodeCtx::Deterministic) {
            ctx.try_check(|| array_elements_are_deterministic(&element_bytes))?;
        }

        elements.push(element_bytes);
    }

    Ok(elements)
}

/// Validates that a CBOR array element follows the deterministic encoding rules as
/// specified in RFC 8949. In this case, it validates that the elements themselves must be
/// deterministically encoded (4.2.1).
fn array_elements_are_deterministic(element_bytes: &[u8]) -> Result<(), minicbor::decode::Error> {
    // if the array elements are not a txt string or byte string we cannot get a declared
    // length
    if let Some(element_declared_length) = get_declared_length(element_bytes)? {
        let header_size = get_cbor_header_size(element_bytes)?;
        let actual_content_size =
            element_bytes
                .len()
                .checked_sub(header_size)
                .ok_or_else(|| {
                    minicbor::decode::Error::message("Integer overflow in content size calculation")
                })?;

        if element_declared_length != actual_content_size {
            return Err(minicbor::decode::Error::message(
                "Declared length does not match the actual length. Non deterministic array element.",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use minicbor::{Decode, Decoder};

    use super::*;

    /// Ensures that encoding and decoding an array preserves:
    /// - The exact byte representation of elements
    /// - The definite length encoding format
    /// - The order of elements
    #[test]
    fn test_array_bytes_roundtrip() {
        // Create a valid deterministic array encoding
        let valid_array = vec![
            0x82, // Array with 2 elements
            0x41, 0x01, // Element 1: 1-byte string
            0x42, 0x01, 0x02, // Element 2: 2-byte string
        ];

        let mut decoder = Decoder::new(&valid_array);
        let result = Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).unwrap();

        // Verify we got back exactly the same bytes
        assert_eq!(
            result,
            Array(vec![
                vec![0x41, 0x01],       // Element 1: 1-byte string
                vec![0x42, 0x01, 0x02], // Element 2: 2-byte string
            ])
        );
    }

    /// Test empty array handling - special case mentioned in RFC 8949.
    /// An empty array is valid and must still follow length encoding rules
    /// from Section 4.2.1.
    #[test]
    fn test_empty_array() {
        let empty_array = vec![
            0x80, // Array with 0 elements - encoded with immediate value as per Section 4.2.1
        ];
        let mut decoder = Decoder::new(&empty_array);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());
    }

    /// Test minimal length encoding rules for arrays as specified in RFC 8949 Section
    /// 4.2.1
    ///
    /// From RFC 8949 Section 4.2.1:
    /// "The length of arrays, maps, strings, and byte strings must be encoded in the
    /// smallest possible way. For arrays (major type 4), lengths 0-23 must be encoded
    /// in the initial byte."
    #[test]
    fn test_array_minimal_length_encoding() {
        // Test case 1: Valid minimal encoding (length = 1)
        let valid_small = vec![
            0x81, // Array, length 1 (major type 4 with immediate value 1)
            0x01, // Element: unsigned int 1
        ];
        let mut decoder = Decoder::new(&valid_small);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());

        // Test case 2: Invalid non-minimal encoding (using additional info 24 for length 1)
        let invalid_small = vec![
            0x98, // Array with additional info = 24 (0x80 | 0x18)
            0x01, // Length encoded as uint8 = 1
            0x01, // Element: unsigned int 1
        ];
        let mut decoder = Decoder::new(&invalid_small);
        assert!(Array::decode(&mut decoder.clone(), &mut DecodeCtx::Deterministic).is_err());
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::non_deterministic()).is_ok());
    }

    /// Test handling of complex element structures while maintaining deterministic
    /// encoding
    ///
    /// RFC 8949 Section 4.2 requires that all elements be deterministically encoded:
    /// "All contained items must also follow the same rules."
    #[test]
    fn test_array_complex_elements() {
        // Test nested structures in elements
        let valid_complex = vec![
            0x83, // Array with 3 elements
            0x41, 0x01, // Element 1: simple 1-byte string
            0x42, 0x01, 0x02, // Element 2: 2-byte string
            0x43, 0x01, 0x02, 0x03, // Element 3: 3-byte string
        ];
        let mut decoder = Decoder::new(&valid_complex);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());
    }

    /// Test edge cases for array encoding while maintaining compliance with RFC 8949
    ///
    /// These cases test boundary conditions that must still follow all rules from
    /// Section 4.2:
    /// - Minimal length encoding (4.2.1)
    /// - No indefinite lengths (4.2.2)
    /// - Deterministic element encoding
    #[test]
    fn test_array_edge_cases() {
        // Single element array - must still follow minimal length encoding rules
        let single_element = vec![
            0x81, // Array with 1 element (using immediate value as per Section 4.2.1)
            0x41, 0x01, // Element: 1-byte string
        ];
        let mut decoder = Decoder::new(&single_element);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());

        // Array with zero-length string element - tests smallest possible element case
        let zero_length_element = vec![
            0x81, // Array with 1 element
            0x40, // Element: 0-byte string (smallest possible element)
        ];
        let mut decoder = Decoder::new(&zero_length_element);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());
    }

    /// Test array with multiple elements of different types
    #[test]
    fn test_array_mixed_elements() {
        // Array with integer, string, and nested array elements
        let mixed_array = vec![
            0x83, // Array with 3 elements
            0x01, // Element 1: unsigned int 1
            0x41, 0x48, // Element 2: 1-byte string "H"
            0x81, 0x02, // Element 3: nested array with one element (unsigned int 2)
        ];
        let mut decoder = Decoder::new(&mixed_array);
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::Deterministic).is_ok());
    }

    /// Test array with multiple elements
    #[test]
    fn test_array_larger_size() {
        // Test with a simple array of 5 single-byte strings
        let array_5 = vec![
            0x85, // Array with 5 elements
            0x41, 0x01, // Element 1: 1-byte string with value 0x01
            0x41, 0x02, // Element 2: 1-byte string with value 0x02
            0x41, 0x03, // Element 3: 1-byte string with value 0x03
            0x41, 0x04, // Element 4: 1-byte string with value 0x04
            0x41, 0x05, // Element 5: 1-byte string with value 0x05
        ];

        let mut decoder = Decoder::new(&array_5);
        let result = Array::decode(&mut decoder, &mut DecodeCtx::Deterministic);
        assert!(result.is_ok());

        let array = result.unwrap();
        assert_eq!(array.len(), 5);

        // Verify the elements are correctly decoded
        assert_eq!(array[0], vec![0x41, 0x01]);
        assert_eq!(array[1], vec![0x41, 0x02]);
        assert_eq!(array[2], vec![0x41, 0x03]);
        assert_eq!(array[3], vec![0x41, 0x04]);
        assert_eq!(array[4], vec![0x41, 0x05]);
    }

    /// Test indefinite-length array rejection in deterministic mode
    /// and acceptance in non-deterministic mode
    #[test]
    fn test_indefinite_length_array_rejection() {
        // Indefinite-length array (not allowed in deterministic encoding)
        let indefinite_array = vec![
            0x9F, // Array with indefinite length
            0x01, // Element 1
            0x02, // Element 2
            0xFF, // Break code
        ];
        let mut decoder = Decoder::new(&indefinite_array);
        assert!(Array::decode(&mut decoder.clone(), &mut DecodeCtx::Deterministic).is_err());
        // Should work in non-deterministic mode
        assert!(Array::decode(&mut decoder, &mut DecodeCtx::non_deterministic()).is_ok());
    }
}
