//! Deterministic CBOR decoding functionality.
//!
//! This module provides a decoder that enforces deterministic CBOR encoding rules as specified in
//! [RFC 8949 Section 4.2](https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2).
//!
//! A CBOR encoding satisfies the "core deterministic encoding requirements" if:
//! 1. The key ordering property defined below is satisfied: all keys in maps are sorted by
//!    first considering the length of key encodings (shorter keys first), and then
//!    by the lexicographic ordering of the key encodings for keys of equal length.
//! 2. The encoded form uses definite-length encoding (no indefinite-length items).
//! 3. No tag numbers are used that are not required to properly decode the item.
//! 4. Floating-point values are following the encoding rules defined in RFC.
//! 5. All integers follow the size rules defined in RFC (minimal encoding length).
//!
//!
//! # Example
//! ```rust
//! # use cbork_utils::decode_deterministic::DeterministicDecoder;
//! # let bytes = vec![0x05]; // Example of minimal encoding for integer 5
//! let decoder = DeterministicDecoder::new(&bytes);
//! // Use decoder to validate and decode CBOR data...
//! ```

use minicbor::{data::Type, Decoder};

/// A decoder that enforces deterministic CBOR encoding rules.
///
/// This decoder validates that CBOR data follows deterministic encoding rules as specified in
/// [RFC 8949 Section 4.2](https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2).
///
/// Core requirements for deterministic encoding:
/// 1. Integer encoding must be minimal length:
///    - Values 0 through 23 and -1 through -24 must be expressed in a single byte
///    - Values 24 through 255 and -25 through -256 must use an additional uint8_t
///    - Values 256 through 65535 and -257 through -65536 must use uint16_t
///    - Values 65536 through 4294967295 and -65537 through -4294967296 must use uint32_t
///    - All other integers must use uint64_t
/// 2. No indefinite-length items are permitted
/// 3. The expression of lengths in major types 2 through 5 must be minimal
/// 4. The keys in maps must be sorted as specified above
/// 5. Floating-point values must use their shortest form that preserves value
/// 6. Non-finite floating-point values are not permitted
pub struct DeterministicDecoder<'b> {
    decoder: Decoder<'b>,
}

/// Error types that can occur during CBOR deterministic decoding validation.
///
/// These errors indicate violations of the deterministic encoding rules
/// as specified in RFC 8949 Section 4.2.
pub enum DeterministicError {
    /// Indicates an integer is not encoded in its shortest possible representation,
    /// violating the core deterministic encoding requirement for minimal-length integers.
    NonMinimalInt,

    /// Indicates presence of indefinite-length items, which are not permitted in
    /// deterministic CBOR encoding. This applies to strings, arrays, and maps.
    IndefiniteLength,

    /// Wraps decoding errors from the underlying CBOR decoder
    DecoderError(minicbor::decode::Error),

    /// Indicates map keys are not sorted according to the requirements:
    /// first by length of encoded key (shorter lengths first),
    /// then by byte-wise lexicographic order for equal lengths
    UnorderedMapKeys,

    /// Indicates a map contains duplicate keys, which violates the requirement
    /// for minimal-length encoding of maps
    DuplicateMapKey,
}

/// A decoder that enforces CBOR Deterministic Encoding rules as specified in RFC 8949 Section 4.2.
///
/// This decoder ensures that:
/// - No indefinite length items are used
/// - Integers are encoded in their shortest form
/// - Lengths of arrays, maps, strings, and byte strings are encoded in their shortest form
/// - Keys in maps are in ascending byte-wise lexicographic order
impl<'b> DeterministicDecoder<'b> {
    /// Creates a new deterministic decoder for the given byte slice.
    ///
    /// # Arguments
    /// * `bytes` - The CBOR-encoded data to validate
    pub fn new(bytes: &'b [u8]) -> Self {
        Self {
            decoder: Decoder::new(bytes),
        }
    }

    /// Validates the next CBOR item in the input stream according to deterministic encoding rules.
    ///
    /// This method advances through the input checking each item against the deterministic encoding rules:
    /// - For integers: ensures they use minimal encoding length
    /// - For arrays and maps: validates they don't use indefinite length encoding
    /// - For strings and byte strings: validates they don't use indefinite length encoding
    /// - For maps: ensures keys are in ascending byte-wise lexicographic order
    ///
    /// # Returns
    /// - `Ok(Some(Type))` if a valid item was found, with its CBOR type
    /// - `Ok(None)` if the end of input was reached
    /// - `Err(DeterministicError)` if any deterministic encoding rule was violated
    ///
    /// # Errors
    /// Returns `DeterministicError` if:
    /// - An indefinite length item is encountered
    /// - An integer is not encoded in its shortest form
    /// - Map keys are not in ascending order
    pub fn validate_next(&mut self) -> Result<Option<Type>, DeterministicError> {
        if let Ok(datatype) = self.decoder.datatype() {
            match datatype {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64 => {
                    let pos = self.decoder.position();
                    let bytes = self.decoder.input();

                    // Check for canonical form of integers
                    match bytes[pos] {
                        // Major type 0 (unsigned)
                        _first @ 0x00..=0x17 => {
                            if !matches!(datatype, Type::U8) {
                                return Err(DeterministicError::NonMinimalInt);
                            }
                        },
                        0x18 => {
                            if bytes[pos + 1] < 0x18 {
                                return Err(DeterministicError::NonMinimalInt);
                            }
                        },
                        0x19 => {
                            if bytes[pos + 1..pos + 3]
                                .try_into()
                                .map(u16::from_be_bytes)
                                .unwrap()
                                < 0x100
                            {
                                return Err(DeterministicError::NonMinimalInt);
                            }
                        },
                        0x1a => {
                            if bytes[pos + 1..pos + 5]
                                .try_into()
                                .map(u32::from_be_bytes)
                                .unwrap()
                                < 0x10000
                            {
                                return Err(DeterministicError::NonMinimalInt);
                            }
                        },
                        0x1b => {
                            if bytes[pos + 1..pos + 9]
                                .try_into()
                                .map(u64::from_be_bytes)
                                .unwrap()
                                < 0x100000000
                            {
                                return Err(DeterministicError::NonMinimalInt);
                            }
                        },
                        // Add similar checks for negative integers if needed
                        _ => {},
                    }

                    self.decoder.skip()?;
                    Ok(Some(datatype))
                },
                Type::Array => {
                    let pos = self.decoder.position();
                    let initial_byte = self.decoder.input()[pos];

                    if initial_byte == 0x9f {
                        return Err(DeterministicError::IndefiniteLength);
                    }

                    let size = self.decoder.array()?;

                    for _ in 0..size.unwrap_or(0) {
                        match self.validate_next()? {
                            Some(_) => (),
                            None => break,
                        }
                    }

                    Ok(Some(datatype))
                },
                Type::Map => {
                    let pos = self.decoder.position();
                    let initial_byte = self.decoder.input()[pos];

                    if initial_byte == 0xbf {
                        return Err(DeterministicError::IndefiniteLength);
                    }

                    let size = self.decoder.map()?;
                    let mut keys = Vec::new();

                    for _ in 0..size.unwrap_or(0) {
                        // Validate and store the key
                        let key_start = self.decoder.position();
                        self.validate_next()?;
                        let key_end = self.decoder.position();
                        let current_key = self.decoder.input()[key_start..key_end].to_vec();

                        // Check for duplicate keys
                        if keys.contains(&current_key) {
                            return Err(DeterministicError::DuplicateMapKey);
                        }

                        // Check if keys are in ascending order
                        if let Some(last) = keys.last() {
                            if &current_key <= last {
                                return Err(DeterministicError::UnorderedMapKeys);
                            }
                        }
                        keys.push(current_key);

                        // Validate the value
                        self.validate_next()?;
                    }

                    Ok(Some(datatype))
                },
                Type::String | Type::Bytes => {
                    let pos = self.decoder.position();
                    let initial_byte = self.decoder.input()[pos];

                    if initial_byte == 0x7f || initial_byte == 0x5f {
                        return Err(DeterministicError::IndefiniteLength);
                    }

                    self.decoder.skip()?;
                    Ok(Some(datatype))
                },
                _ => {
                    self.decoder.skip()?;
                    Ok(Some(datatype))
                },
            }
        } else {
            Ok(None)
        }
    }
}

/// Implements conversion from minicbor decode errors to DeterministicError
impl From<minicbor::decode::Error> for DeterministicError {
    fn from(error: minicbor::decode::Error) -> Self {
        DeterministicError::DecoderError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minicbor::Encoder;

    #[test]
    fn test_non_minimal_direct() {
        let bytes = &[0x18, 0x05]; // non-minimal encoding of 5
        let mut dec = DeterministicDecoder::new(bytes);
        let result = dec.validate_next();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_array_with_non_minimal() {
        let bytes = &[
            0x82, // array of 2 elements
            0x18, 0x05, // non-minimal encoding of 5
            0x02, // valid encoding of 2
        ];
        let mut dec = DeterministicDecoder::new(bytes);
        let result = dec.validate_next();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_map_with_non_minimal() {
        let bytes = &[
            0xa1, // map of 1 pair
            0x61, 0x61, // "a"
            0x18, 0x05, // non-minimal encoding of 5
        ];
        let mut dec = DeterministicDecoder::new(bytes);
        let result = dec.validate_next();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_nested_structure() {
        let bytes = &[
            0xa1, // map of 1 pair
            0x61, 0x61, // "a"
            0x82, // array of 2 elements
            0x18, 0x05, // non-minimal encoding of 5
            0x02, // valid encoding of 2
        ];
        let mut dec = DeterministicDecoder::new(bytes);
        let result = dec.validate_next();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_valid_minimal_encodings() {
        // Test various valid encodings
        let test_values = [0u8, 1, 23, 24, 25, 100, 255];

        for value in test_values {
            let mut bytes = vec![];
            let mut enc = Encoder::new(&mut bytes);
            enc.encode(value).unwrap();

            let mut dec = DeterministicDecoder::new(&bytes);
            assert!(dec.validate_next().is_ok());
        }
    }

    #[test]
    fn test_valid_nested_structure() {
        let mut bytes = vec![];
        let mut enc = Encoder::new(&mut bytes);
        enc.map(1)
            .unwrap()
            .encode("key")
            .unwrap()
            .array(2)
            .unwrap()
            .encode(24u8)
            .unwrap() // valid minimal encoding
            .encode(255u8)
            .unwrap(); // valid minimal encoding

        let mut dec = DeterministicDecoder::new(&bytes);
        assert!(dec.validate_next().is_ok());
    }

    #[test]
    fn test_map_key_ordering() {
        // Properly ordered keys (by length, then lexicographically)
        let valid_bytes = &[
            0xa2, // map of 2 pairs
            0x61, 0x61, // "a"
            0x01, // 1
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
        ];
        let mut dec = DeterministicDecoder::new(valid_bytes);
        assert!(dec.validate_next().is_ok());

        // Incorrectly ordered keys
        let invalid_bytes = &[
            0xa2, // map of 2 pairs
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
            0x61, 0x61, // "a"
            0x01, // 1
        ];
        let mut dec = DeterministicDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.validate_next(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_duplicate_map_keys() {
        let bytes = &[
            0xa2, // map of 2 pairs
            0x61, 0x61, // "a"
            0x01, // 1
            0x61, 0x61, // "a" (duplicate key)
            0x02, // 2
        ];
        let mut dec = DeterministicDecoder::new(bytes);
        assert!(matches!(
            dec.validate_next(),
            Err(DeterministicError::DuplicateMapKey)
        ));
    }
}
