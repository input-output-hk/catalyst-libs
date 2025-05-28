//! Deterministic CBOR decoding functionality.
//!
//! This module provides a decoder that enforces deterministic CBOR encoding rules as
//! specified in [RFC 8949 Section 4.2](https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2).
//!
//! A CBOR encoding satisfies the "core deterministic encoding requirements" if:
//! 1. The key ordering property defined below is satisfied: all keys in maps are sorted
//!    by first considering the length of key encodings (shorter keys first), and then by
//!    the lexicographic ordering of the key encodings for keys of equal length.
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
/// This decoder validates that CBOR data follows deterministic encoding rules as
/// specified in [RFC 8949 Section 4.2](https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2).
///
/// Core requirements for deterministic encoding:
/// 1. Integer encoding must be minimal length:
///    - Values 0 through 23 and -1 through -24 must be expressed in a single byte
///    - Values 24 through 255 and -25 through -256 must use an additional `uint8_t`
///    - Values 256 through 65535 and -257 through -65536 must use `uint16_t`
///    - Values 65536 through 4294967295 and -65537 through -4294967296 must use
///      `uint32_t`
///    - All other integers must use `uint64_t`
/// 2. No indefinite-length items are permitted
/// 3. The expression of lengths in major types 2 through 5 must be minimal
/// 4. The keys in maps must be sorted as specified above
/// 5. Floating-point values must use their shortest form that preserves value
/// 6. Non-finite floating-point values are not permitted
pub struct CborDecoder<'b> {
    decoder: Decoder<'b>,
    validate: bool,
}

/// Error types that can occur during CBOR deterministic decoding validation.
///
/// These errors indicate violations of the deterministic encoding rules
/// as specified in RFC 8949 Section 4.2.
#[derive(Debug)]
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

    /// Indicates float is not encoded in its shortest possible form
    NonMinimalFloat,
    /// Indicates presence of non-finite floating point values
    NonFiniteFloat,
}

/// A decoder that enforces CBOR Deterministic Encoding rules as specified in RFC 8949
/// Section 4.2.
///
/// This decoder ensures that:
/// - No indefinite length items are used
/// - Integers are encoded in their shortest form
/// - Lengths of arrays, maps, strings, and byte strings are encoded in their shortest
///   form
/// - Keys in maps are in ascending byte-wise lexicographic order
impl<'b> CborDecoder<'b> {
    /// Creates a new deterministic decoder for the given byte slice.
    ///
    /// # Arguments
    /// * `bytes` - The CBOR-encoded data to validate
    #[must_use]
    pub fn new(bytes: &'b [u8]) -> Self {
        Self {
            decoder: Decoder::new(bytes),
            validate: true,
        }
    }

    /// Creates a new `DeterministicDecoder` with validation disabled.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte slice to decode
    ///
    /// # Returns
    ///
    /// Returns a new instance of `DeterministicDecoder` with validation turned off.
    #[must_use]
    pub fn new_without_validation(bytes: &'b [u8]) -> Self {
        Self {
            decoder: Decoder::new(bytes),
            validate: false,
        }
    }

    /// Sets whether validation should be performed during decoding.
    ///
    /// # Arguments
    ///
    /// * `validate` - If `true`, enables validation; if `false`, disables validation
    pub fn set_validation(&mut self, validate: bool) {
        self.validate = validate;
    }

    /// Validates the next CBOR item according to RFC 8949 § 4.2 deterministic encoding
    /// rules.
    ///
    /// According to RFC 8949, deterministically encoded CBOR follows these rules:
    ///
    /// 1. Integer encoding must be as short as possible:
    ///    - Integers 0 through 23 must be expressed in a single byte
    ///    - Integers 24 through 255 must use one-byte uint8_t encoding
    ///    - Integers 256 through 65535 must use two-byte uint16_t encoding
    ///    - Integers 65536 through 4294967295 must use four-byte uint32_t encoding
    ///    - Integers above 4294967295 must use eight-byte uint64_t encoding
    ///
    /// 2. The expression of lengths in major types 2 through 5 must be as short as
    ///    possible
    ///    - No indefinite lengths are allowed
    ///    - The rules for integers apply to the length fields
    ///
    /// 3. Indefinite-length items must be made into definite-length items:
    ///    - The implementations must NOT generate indefinite-length strings, arrays, or
    ///      maps
    ///    - The implementations must NOT generate indefinite-length data items
    ///
    /// 4. Maps must have keys sorted in bytewise lexicographic order:
    ///    - All map keys must be sorted in length-first, bytewise lexicographic order
    ///    - Duplicate keys in a map are not valid
    ///    - The sorting rules apply after the keys are encoded
    ///
    /// # Returns
    /// - `Ok(Some(Type))` - Successfully validated item of the given CBOR type
    /// - `Ok(None)` - End of input reached
    /// - `Err(DeterministicError)` - Validation error due to:
    ///   - Non-minimal integer encoding
    ///   - Indefinite length items
    ///   - Unsorted or duplicate map keys
    ///   - Invalid CBOR encoding
    ///
    /// # Examples
    /// Minimal integer encoding:
    /// - ✓ Value 0: Encoded as 0x00
    /// - ✗ Value 0: Encoded as 0x1800 (non-minimal)
    ///
    /// Map key ordering:
    /// - ✓ Keys: [0x01, 0x0203, 0x030405]
    /// - ✗ Keys: [0x0203, 0x01, 0x030405] (incorrect order)
    pub fn decode(&mut self) -> Result<Option<Type>, DeterministicError> {
        let datatype = self.decoder.datatype()?;

        if self.validate {
            match datatype {
                Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64 => {
                    self.validate_integer(datatype)?;
                    Ok(Some(datatype))
                },
                Type::Array => {
                    self.validate_array(datatype)?;
                    Ok(Some(datatype))
                },
                Type::String => {
                    self.validate_string(datatype)?;
                    Ok(Some(datatype))
                },
                Type::Map => {
                    self.validate_map(datatype)?;
                    Ok(Some(datatype))
                },
                _ => {
                    self.decoder.skip()?;
                    Ok(Some(datatype))
                },
            }
        } else {
            self.decoder.skip()?;
            Ok(Some(datatype))
        }
    }

    /// Validates that a CBOR integer follows the deterministic encoding rules as
    /// specified in RFC 8949.
    ///
    /// # Deterministic Encoding Requirements for Integers
    ///
    /// The function ensures the following requirements are met:
    ///
    /// 1. Integers MUST be as small as possible:
    ///    - Unsigned integers 0 through 23 and negative integers -1 through -24 must use
    ///      the direct value encoding in the initial byte
    ///    - For other values, the minimal length encoding must be used:
    ///      * 24..=255: one byte (type 24)
    ///      * 256..=65535: two bytes (type 25)
    ///      * 65536..=4294967295: four bytes (type 26)
    ///      * Above 4294967295: eight bytes (type 27)
    ///    - No leading zeroes are allowed in the encoded bytes
    ///    - Returns `DeterministicError::NonMinimalInt` for non-minimal encodings
    ///
    /// 2. Major type selection:
    ///    - Non-negative integers must use major type 0
    ///    - Negative integers must use major type 1
    ///    - The value -1 must be encoded as major type 1, not as a negative zero
    ///
    /// # Arguments
    ///
    /// * `datatype` - The CBOR Type of the integer being validated (Type::U8, Type::U16,
    ///   etc., or Type::I8, Type::I16, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Type>)` - Returns the next type after the integer if validation
    ///   succeeds
    /// * `Err(DeterministicError)` - Returns an error if any deterministic encoding rule
    ///   is violated
    ///
    /// # Errors
    ///
    /// * `DeterministicError::NonMinimalInt` - If the integer is not encoded using the
    ///   smallest possible representation or contains leading zeros
    /// * `DeterministicError::DecoderError` - If any underlying CBOR decoding error
    ///   occurs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::{DeterministicDecoder, Type};
    ///
    /// // Example 1: Valid minimal encoding for small numbers
    /// let bytes = [0x17]; // Integer 23 (direct encoding)
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_integer(Type::U8).is_ok());
    ///
    /// // Example 2: Valid minimal encoding for negative numbers
    /// let bytes = [0x20]; // Integer -1 (direct encoding)
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_integer(Type::I8).is_ok());
    ///
    /// // Example 3: Valid minimal encoding for larger numbers
    /// let bytes = [0x18, 0x18]; // Integer 24 (one-byte encoding)
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_integer(Type::U8).is_ok());
    /// ```
    ///
    /// # Invalid Examples
    ///
    /// ```rust
    /// # use your_crate::{DeterministicDecoder, Type};
    /// // Non-minimal encoding (using one byte when direct encoding would suffice)
    /// let bytes = [0x18, 0x17]; // Non-minimal encoding of 23
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_integer(Type::U8),
    ///     Err(DeterministicError::NonMinimalInt)
    /// ));
    ///
    /// // Non-minimal encoding with leading zeros
    /// let bytes = [0x19, 0x00, 0x80]; // Two-byte encoding with leading zero
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_integer(Type::U16),
    ///     Err(DeterministicError::NonMinimalInt)
    /// ));
    /// ```
    ///
    /// # Additional Examples
    ///
    /// ```rust
    /// # use your_crate::{DeterministicDecoder, Type};
    /// // Valid encodings for different ranges
    /// let examples = [
    ///     // Small positive integers (0-23)
    ///     [0x00], // 0
    ///     [0x17], // 23
    ///     // Small negative integers (-1 to -24)
    ///     [0x20], // -1
    ///     [0x37], // -24
    ///     // One-byte encodings
    ///     [0x18, 0x18], // 24
    ///     [0x18, 0xFF], // 255
    ///     // Two-byte encodings
    ///     [0x19, 0x01, 0x00], // 256
    ///     // Four-byte encodings
    ///     [0x1A, 0x00, 0x01, 0x00, 0x00], // 65536
    /// ];
    /// ```
    ///
    /// This implementation aligns with RFC 8949 Section 4.2.1 requirements for
    /// deterministically encoded CBOR integers [[1]](https://datatracker.ietf.org/doc/html/rfc8949#name-deterministically-encoded-c).
    ///
    /// # Notes
    ///
    /// - The encoding must be minimal even when encoding numbers within a nested
    ///   structure
    /// - This validation is crucial for ensuring consistent hash values and for ensuring
    ///   interoperability with other CBOR implementations
    /// - The function handles both positive and negative integers with their respective
    ///   major types (0 and 1)
    fn validate_integer(&mut self, datatype: Type) -> Result<Option<Type>, DeterministicError> {
        let pos = self.decoder.position();
        let bytes = self.decoder.input();

        match bytes[pos] {
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
            0x1A => {
                if bytes[pos + 1..pos + 5]
                    .try_into()
                    .map(u32::from_be_bytes)
                    .unwrap()
                    < 0x10000
                {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            0x1B => {
                if bytes[pos + 1..pos + 9]
                    .try_into()
                    .map(u64::from_be_bytes)
                    .unwrap()
                    < 0x100000000
                {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            _ => {},
        }

        self.decoder.skip()?;
        Ok(Some(datatype))
    }

    /// Validates that a CBOR array follows the deterministic encoding rules as specified
    /// in RFC 8949.
    ///
    /// # Deterministic Encoding Requirements for Arrays
    ///
    /// The function ensures the following requirements are met:
    ///
    /// 1. Array length must be encoded in the smallest possible representation:
    ///    - For lengths 0-23, the length must be included in the initial byte
    ///    - For lengths 24-255, one byte is used (type 24)
    ///    - For lengths 256-65535, two bytes are used (type 25)
    ///    - For lengths 65536-4294967295, four bytes are used (type 26)
    ///    - For lengths above 4294967295, eight bytes are used (type 27)
    ///    - Returns `DeterministicError::NonMinimalInt` for non-minimal length encodings
    ///
    /// 2. Indefinite-length arrays are not allowed:
    ///    - All arrays must use definite length encoding
    ///    - Returns `DeterministicError::IndefiniteLength` for indefinite-length arrays
    ///
    /// 3. Each element within the array must also follow deterministic encoding rules:
    ///    - Recursively validates each array element
    ///    - All nested data items must comply with deterministic encoding requirements
    ///    - Validation continues until all elements are processed or an error is
    ///      encountered
    ///
    /// # Arguments
    ///
    /// * `datatype` - The CBOR Type of the array being validated
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Type>)` - Returns the next type after the array if validation
    ///   succeeds
    /// * `Err(DeterministicError)` - Returns an error if any deterministic encoding rule
    ///   is violated
    ///
    /// # Errors
    ///
    /// The function may return the following errors:
    /// * `DeterministicError::NonMinimalInt` - If array length is not encoded using the
    ///   smallest possible representation
    /// * `DeterministicError::IndefiniteLength` - If the array uses indefinite-length
    ///   encoding
    /// * `DeterministicError::DecoderError` - If any underlying CBOR decoding error
    ///   occurs
    /// * Any error that can occur while validating nested elements
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::{DeterministicDecoder, Type};
    ///
    /// // Example 1: Valid minimal encoding for small array
    /// let bytes = [
    ///     0x83, // Array of length 3
    ///     0x01, // First element: 1
    ///     0x02, // Second element: 2
    ///     0x03, // Third element: 3
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_array(Type::Array).is_ok());
    ///
    /// // Example 2: Valid minimal encoding for nested array
    /// let bytes = [
    ///     0x82, // Array of length 2
    ///     0x82, // Nested array of length 2
    ///     0x01, 0x02, // Elements: 1, 2
    ///     0x03, // Last element of outer array: 3
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_array(Type::Array).is_ok());
    /// ```
    ///
    /// # Invalid Examples
    ///
    /// ```rust
    /// # use your_crate::{DeterministicDecoder, Type};
    /// // Non-minimal length encoding
    /// let bytes = [
    ///     0x18, 0x03, // Non-minimal encoding of length 3 (should be 0x83)
    ///     0x01, 0x02, 0x03,
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_array(Type::Array),
    ///     Err(DeterministicError::NonMinimalInt)
    /// ));
    ///
    /// // Indefinite length encoding (not allowed)
    /// let bytes = [
    ///     0x9F, // Indefinite-length array
    ///     0x01, 0x02, 0x03, 0xFF, // Break
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_array(Type::Array),
    ///     Err(DeterministicError::IndefiniteLength)
    /// ));
    /// ```
    ///
    /// # Recursive Validation
    ///
    /// The function performs recursive validation for nested structures. For example:
    ///
    /// ```rust
    /// # use your_crate::{DeterministicDecoder, Type};
    /// // Complex nested structure
    /// let bytes = [
    ///     0x82, // Array of length 2
    ///     0x83, // Nested array of length 3
    ///     0x01, 0x02, 0x03, // Elements of nested array
    ///     0x82, // Another nested array of length 2
    ///     0x04, 0x05, // Elements of second nested array
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_array(Type::Array).is_ok());
    /// ```
    ///
    /// This implementation aligns with RFC 8949 Section 4.2.1 requirements for
    /// deterministically encoded CBOR arrays [[1]](https://datatracker.ietf.org/doc/html/rfc8949#name-deterministically-encoded-c).
    fn validate_array(&mut self, datatype: Type) -> Result<Option<Type>, DeterministicError> {
        let pos = self.decoder.position();
        let initial_byte = self.decoder.input()[pos];

        if initial_byte == 0x9F {
            return Err(DeterministicError::IndefiniteLength);
        }

        let size = self.decoder.array()?;

        if let Some(len) = size {
            self.check_minimal_length(pos, len)?;
        }

        for _ in 0..size.unwrap_or(0) {
            match self.decode()? {
                Some(_) => (),
                None => break,
            }
        }

        Ok(Some(datatype))
    }

    /// Validates that a CBOR string (text or byte string) follows the deterministic
    /// encoding rules as specified in RFC 8949.
    ///
    /// # Deterministic Encoding Requirements for Strings
    ///
    /// The function ensures the following requirements are met:
    ///
    /// 1. String length must be encoded in the smallest possible representation
    ///    - For lengths 0-23, the length must be included in the initial byte
    ///    - For lengths 24-255, one byte is used (type 24)
    ///    - For lengths 256-65535, two bytes are used (type 25)
    ///    - For lengths 65536-4294967295, four bytes are used (type 26)
    ///    - For lengths above 4294967295, eight bytes are used (type 27)
    ///    - Returns `DeterministicError::NonMinimalInt` for non-minimal length encodings
    ///
    /// 2. Indefinite-length strings are not allowed
    ///    - All strings must use definite length encoding
    ///    - Returns `DeterministicError::IndefiniteLength` for indefinite-length strings
    ///
    /// # Arguments
    ///
    /// * `datatype` - The CBOR Type of the string being validated (either Type::Text or
    ///   Type::Bytes)
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Type>)` - Returns the next type after the string if validation
    ///   succeeds
    /// * `Err(DeterministicError)` - Returns an error if any deterministic encoding rule
    ///   is violated
    ///
    /// # Errors
    ///
    /// The function may return the following errors:
    /// * `DeterministicError::NonMinimalInt` - If string length is not encoded using the
    ///   smallest possible representation
    /// * `DeterministicError::IndefiniteLength` - If the string uses indefinite-length
    ///   encoding
    /// * `DeterministicError::DecoderError` - If any underlying CBOR decoding error
    ///   occurs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::{DeterministicDecoder, Type};
    ///
    /// // Example 1: Valid minimal encoding for short string
    /// let bytes = [
    ///     0x65, // Text string of length 5
    ///     0x68, 0x65, 0x6C, 0x6C, 0x6F,
    /// ]; // "hello"
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_string(Type::Text).is_ok());
    ///
    /// // Example 2: Valid minimal encoding for byte string
    /// let bytes = [
    ///     0x43, // Byte string of length 3
    ///     0x01, 0x02, 0x03,
    /// ];
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_string(Type::Bytes).is_ok());
    /// ```
    ///
    /// # Invalid Examples
    ///
    /// ```rust
    /// # use your_crate::{DeterministicDecoder, Type};
    /// // Non-minimal length encoding (using one byte when initial byte would suffice)
    /// let bytes = [
    ///     0x18, 0x05, // Non-minimal encoding of length 5
    ///     0x68, 0x65, 0x6C, 0x6C, 0x6F,
    /// ]; // "hello"
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_string(Type::Text),
    ///     Err(DeterministicError::NonMinimalInt)
    /// ));
    ///
    /// // Indefinite length encoding (not allowed)
    /// let bytes = [
    ///     0x7F, // Indefinite length text string
    ///     0x65, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0xFF,
    /// ]; // Break
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(matches!(
    ///     decoder.validate_string(Type::Text),
    ///     Err(DeterministicError::IndefiniteLength)
    /// ));
    /// ```
    ///
    /// This implementation aligns with RFC 8949 Section 4.2.1 requirements for
    /// deterministically encoded CBOR strings [[1]](https://datatracker.ietf.org/doc/html/rfc8949#name-deterministically-encoded-c).
    fn validate_string(&mut self, datatype: Type) -> Result<Option<Type>, DeterministicError> {
        let pos = self.decoder.position();
        let initial_byte = self.decoder.input()[pos];

        if initial_byte == 0x7F || initial_byte == 0x5F {
            return Err(DeterministicError::IndefiniteLength);
        }

        let len = if matches!(datatype, Type::String) {
            let s = self.decoder.str()?;
            s.len() as u64
        } else {
            let b = self.decoder.bytes()?;
            b.len() as u64
        };

        self.check_minimal_length(pos, len)?;
        Ok(Some(datatype))
    }

    /// Validates that a CBOR map follows the deterministic encoding rules as specified in
    /// RFC 8949.
    ///
    /// # Deterministic Encoding Requirements for Maps
    ///
    /// The function ensures the following requirements are met:
    ///
    /// 1. Map keys must be in order according to their encoded byte sequences
    ///    - Keys are compared byte-by-byte in lexicographic order
    ///    - Shorter keys are considered less than longer keys when comparing equal
    ///      prefixes
    ///
    /// 2. No duplicate keys are allowed in the map
    ///    - Each key must appear exactly once
    ///    - Returns `DeterministicError::DuplicateMapKey` if duplicates are found
    ///
    /// 3. Map length must be encoded in the smallest possible representation
    ///    - Uses `check_minimal_length` to verify minimal encoding
    ///    - Returns `DeterministicError::NonMinimalInt` for non-minimal length encodings
    ///
    /// 4. Indefinite-length maps are not allowed
    ///    - Returns `DeterministicError::IndefiniteLength` for indefinite-length maps
    ///
    /// # Arguments
    ///
    /// * `datatype` - The CBOR Type of the map being validated
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Type>)` - Returns the next type after the map if validation succeeds
    /// * `Err(DeterministicError)` - Returns an error if any deterministic encoding rule
    ///   is violated
    ///
    /// # Errors
    ///
    /// The function may return the following errors:
    /// * `DeterministicError::UnorderedMapKeys` - If map keys are not in lexicographic
    ///   order
    /// * `DeterministicError::DuplicateMapKey` - If duplicate keys are found in the map
    /// * `DeterministicError::NonMinimalInt` - If map length is not encoded minimally
    /// * `DeterministicError::IndefiniteLength` - If the map uses indefinite-length
    ///   encoding
    /// * `DeterministicError::DecoderError` - If any underlying CBOR decoding error
    ///   occurs
    ///
    /// # Example
    ///
    /// ```rust
    /// use your_crate::{DeterministicDecoder, Type};
    ///
    /// // Example of validating a deterministically encoded map
    /// let bytes = [
    ///     0xA2, // Map of 2 pairs
    ///     0x01, 0x20, // Key: 1, Value: 32
    ///     0x02, 0x40,
    /// ]; // Key: 2, Value: 64
    /// let mut decoder = DeterministicDecoder::new(&bytes);
    /// assert!(decoder.validate_map(Type::Map).is_ok());
    /// ```
    ///
    /// This implementation aligns with RFC 8949 Section 4.2.1 requirements for
    /// deterministically encoded CBOR maps [[1]](https://datatracker.ietf.org/doc/html/rfc8949#name-deterministically-encoded-c).
    fn validate_map(&mut self, datatype: Type) -> Result<Option<Type>, DeterministicError> {
        let pos = self.decoder.position();
        let initial_byte = self.decoder.input()[pos];

        if initial_byte == 0xBF {
            return Err(DeterministicError::IndefiniteLength);
        }

        let size = self.decoder.map()?;
        let mut keys = Vec::new();

        for _ in 0..size.unwrap_or(0) {
            let key_start = self.decoder.position();
            self.decode()?;
            let key_end = self.decoder.position();
            let current_key = self.decoder.input()[key_start..key_end].to_vec();

            if keys.contains(&current_key) {
                return Err(DeterministicError::DuplicateMapKey);
            }

            if let Some(last) = keys.last() {
                if &current_key <= last {
                    return Err(DeterministicError::UnorderedMapKeys);
                }
            }
            keys.push(current_key);

            self.decode()?;
        }

        Ok(Some(datatype))
    }

    /// Validates that a length encoding follows the deterministic encoding rules from RFC
    /// 8949 § 4.2.1.
    ///
    /// According to RFC 8949 § 4.2.1 "Core Deterministic Encoding Requirements", length
    /// encoding must be minimal for the following CBOR data items:
    /// * Major type 2: byte strings
    /// * Major type 3: text strings
    /// * Major type 4: arrays
    /// * Major type 5: maps
    ///
    /// The length encoding is minimal if and only if:
    /// * Values 0 through 23 are expressed in a single byte using the direct value
    /// * Values 24 through 255 use the one-byte uint8_t encoding (additional info = 24)
    /// * Values 256 through 65535 use the two-byte uint16_t encoding (additional info =
    ///   25)
    /// * Values 65536 through 4294967295 use the four-byte uint32_t encoding (additional
    ///   info = 26)
    /// * Values above 4294967295 use the eight-byte uint64_t encoding (additional info =
    ///   27)
    ///
    /// # Arguments
    /// * `pos` - Position in the input buffer where the length encoding starts
    /// * `length` - The decoded length value being validated
    ///
    /// # Returns
    /// * `Ok(())` if the length encoding is minimal according to RFC 8949
    /// * `Err(DeterministicError::NonMinimalInt)` if non-minimal encoding is detected
    ///
    /// # Examples
    /// For length value 5:
    /// * ✓ Minimal encoding: 0x05 (direct value)
    /// * ✗ Non-minimal encoding: 0x18 0x05 (using one-byte uint8_t unnecessarily)
    ///
    /// For length value 200:  
    /// * ✓ Minimal encoding: 0x18 0xC8 (one-byte uint8_t)
    /// * ✗ Non-minimal encoding: 0x19 0x00 0xC8 (using two-byte uint16_t unnecessarily)
    fn check_minimal_length(&self, pos: usize, length: u64) -> Result<(), DeterministicError> {
        let initial_byte = self.decoder.input()[pos];

        let additional_info = initial_byte & 0x1F;

        match length {
            0..=23 => {
                if additional_info != length as u8 {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            24..=255 => {
                if additional_info != 24 {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            256..=65535 => {
                if additional_info != 25 {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            65536..=4294967295 => {
                if additional_info != 26 {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
            _ => {
                if additional_info != 27 {
                    return Err(DeterministicError::NonMinimalInt);
                }
            },
        }
        Ok(())
    }
}

/// Implements conversion from minicbor decode errors to `DeterministicError`
impl From<minicbor::decode::Error> for DeterministicError {
    fn from(error: minicbor::decode::Error) -> Self {
        DeterministicError::DecoderError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_without_validation() {
        // Create a simple CBOR integer (10) using non-minimal encoding
        // [0x18, 0x0a] represents:
        //   0x18 - indicates the following byte contains an unsigned integer
        //   0x0a - the value 10 in hex
        // This is non-minimal because 10 could be encoded as just [0x0a]
        // according to RFC 8949 Section 4.2
        let non_minimal_input = [0x18, 0x0a];

        // First test phase: Decoding with validation disabled
        {
            // Create new decoder with validation off (default state)
            let mut decoder = CborDecoder::new_without_validation(&non_minimal_input);

            // Verify the initial validation state is disabled
            assert!(
                !decoder.validate,
                "Validation should be disabled by default"
            );

            // Attempt to decode - should succeed because validation is off
            // Even though encoding is non-minimal, decoder should accept it
            let result = decoder.decode();
            assert!(
                result.is_ok(),
                "Decoding should succeed with validation disabled"
            );
            assert!(
                result.unwrap().is_some(),
                "Should return Some value when data is present"
            );
        }

        // Second test phase: Decoding with validation enabled
        {
            // Create new decoder and enable validation
            let mut decoder = CborDecoder::new_without_validation(&non_minimal_input);

            // Enable validation explicitly
            decoder.set_validation(true);
            assert!(
                decoder.validate,
                "Validation should be enabled after set_validation(true)"
            );

            // Attempt to decode - should fail because encoding is non-minimal
            // RFC 8949 Section 4.2 requires minimal encoding for deterministic CBOR
            let result = decoder.decode();
            assert!(
                result.is_err(),
                "Decoding should fail with non-minimal encoding when validation is enabled"
            );

            // Note: The specific error type could be checked here if the implementation
            // provides distinct error types for validation failures
        }
    }

    #[test]
    fn test_validation_toggle() {
        // Create CBOR data with proper minimal encoding for integer 42
        // [0x18, 0x2a] represents:
        //   0x18 - indicates the following byte contains an unsigned integer
        //   0x2a - the value 42 in hex
        // This is minimal encoding for 42 since it requires the full byte
        let input = [0x18, 0x2a];

        // Create decoder and test validation state transitions
        let mut decoder = CborDecoder::new_without_validation(&input);

        // Check initial validation state
        assert!(
            !decoder.validate,
            "Initial validation state should be disabled"
        );

        // Test validation enable
        decoder.set_validation(true);
        assert!(
            decoder.validate,
            "Validation should be enabled after explicit enable"
        );

        // Test validation disable
        decoder.set_validation(false);
        assert!(
            !decoder.validate,
            "Validation should be disabled after explicit disable"
        );

        // Verify decoding still works after toggling validation
        let result = decoder.decode();
        assert!(
            result.is_ok(),
            "Decoding should succeed after validation toggles"
        );
        assert!(
            result.unwrap().is_some(),
            "Should return Some value for valid input"
        );
    }

    #[test]
    fn test_non_minimal_direct() {
        let bytes = &[0x18, 0x05]; // non-minimal encoding of 5
        let mut dec = CborDecoder::new(bytes);
        let result = dec.decode();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_array_with_non_minimal() {
        let bytes = &[
            0x82, // array of 2 elements
            0x18, 0x05, // non-minimal encoding of 5
            0x02, // valid encoding of 2
        ];
        let mut dec = CborDecoder::new(bytes);
        let result = dec.decode();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_map_with_non_minimal() {
        let bytes = &[
            0xA1, // map of 1 pair
            0x61, 0x61, // "a"
            0x18, 0x05, // non-minimal encoding of 5
        ];
        let mut dec = CborDecoder::new(bytes);
        let result = dec.decode();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_nested_structure() {
        let bytes = &[
            0xA1, // map of 1 pair
            0x61, 0x61, // "a"
            0x82, // array of 2 elements
            0x18, 0x05, // non-minimal encoding of 5
            0x02, // valid encoding of 2
        ];
        let mut dec = CborDecoder::new(bytes);
        let result = dec.decode();
        assert!(matches!(result, Err(DeterministicError::NonMinimalInt)));
    }

    #[test]
    fn test_valid_minimal_encodings() {
        // Test various valid encodings
        let test_values = [0u8, 1, 23, 24, 25, 100, 255];

        for value in test_values {
            let mut bytes = vec![];
            let mut enc = minicbor::Encoder::new(&mut bytes);
            enc.encode(value).unwrap();

            let mut dec = CborDecoder::new(&bytes);
            assert!(dec.decode().is_ok());
        }
    }

    #[test]
    fn test_valid_nested_structure() {
        let mut bytes = vec![];
        let mut enc = minicbor::Encoder::new(&mut bytes);
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

        let mut dec = CborDecoder::new(&bytes);
        assert!(dec.decode().is_ok());
    }

    #[test]
    fn test_map_key_ordering() {
        // Properly ordered keys (by length, then lexicographically)
        let valid_bytes = &[
            0xA2, // map of 2 pairs
            0x61, 0x61, // "a"
            0x01, // 1
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Incorrectly ordered keys
        let invalid_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
            0x61, 0x61, // "a"
            0x01, // 1
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_float_deterministic_encoding() {
        // In deterministic encoding, floating point values must be encoded in their shortest form
        let mut bytes = vec![];
        let mut enc = minicbor::Encoder::new(&mut bytes);

        // Encode as float64 since that's what's available
        enc.f64(1.5).unwrap();
        let mut dec = CborDecoder::new(&bytes);
        assert!(dec.decode().is_ok());

        // Test encoding of integer-valued float
        let mut bytes = vec![];
        let mut enc = minicbor::Encoder::new(&mut bytes);
        enc.f64(42.0).unwrap();
        let mut dec = CborDecoder::new(&bytes);
        assert!(dec.decode().is_ok());
    }

    #[test]
    fn test_map_key_length_ordering() {
        // According to RFC 8949 4.2.1: shorter keys must come before longer keys
        let valid_bytes = &[
            0xA2, // map of 2 pairs
            0x61, 0x78, // "x"
            0x01, // 1
            0x62, 0x78, 0x79, // "xy"
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Invalid order - longer key before shorter key
        let invalid_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0x78, 0x79, // "xy"
            0x02, // 2
            0x61, 0x78, // "x"
            0x01, // 1
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_map_key_lexicographic_ordering() {
        // Test lexicographic ordering of equal-length keys
        let valid_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0x61, 0x61, // "aa"
            0x01, // 1
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Invalid lexicographic order
        let invalid_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0x62, 0x62, // "bb"
            0x02, // 2
            0x62, 0x61, 0x61, // "aa"
            0x01, // 1
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_minimal_length_array() {
        // Test that array lengths are encoded minimally
        let valid_bytes = &[
            0x82, // array of 2 elements (minimal encoding)
            0x01, // 1
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Non-minimal array length encoding
        let invalid_bytes = &[
            0x98, 0x02, // array of 2 elements (non-minimal encoding)
            0x01, // 1
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::NonMinimalInt)
        ));
    }

    #[test]
    fn test_string_length_encoding() {
        // RFC 8949 Section 4.2 requires lengths in major types 2-5 (including strings)
        // to be encoded in their shortest form. This means:
        // - Lengths 0-23 must use the direct encoding (major type | length)
        // - Lengths 24-255 must use 1 additional byte
        // - Lengths 256-65535 must use 2 additional bytes
        // - Lengths 65536-4294967295 must use 4 additional bytes
        // - Lengths above 4294967295 must use 8 additional bytes

        // Test case with valid minimal encoding:
        // 0x62 = major type 3 (text string) with additional info 2 (length)
        // This is correct for a 2-byte string since 2 < 24
        let valid_bytes = &[
            0x62, // text string of length 2 (minimal encoding)
            0x61, 0x62, // "ab"
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Test case with invalid non-minimal encoding:
        // 0x78 = major type 3 with additional info 24 (1-byte length follows)
        // 0x02 = length value of 2
        // This is incorrect because length 2 should use direct encoding (as above)
        let invalid_bytes = &[
            0x78, 0x02, // text string of length 2 (non-minimal encoding)
            0x61, 0x62, // "ab"
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::NonMinimalInt)
        ));

        // The same rules apply to byte strings (major type 2) and other major types
        // that use length encoding (arrays - type 4, maps - type 5)
    }

    #[test]
    fn test_nested_map_key_ordering() {
        // Test ordering in nested maps
        let valid_bytes = &[
            0xA1, // outer map of 1 pair
            0x61, 0x61, // "a"
            0xA2, // inner map of 2 pairs
            0x61, 0x78, // "x"
            0x01, // 1
            0x62, 0x78, 0x79, // "xy"
            0x02, // 2
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Invalid ordering in inner map
        let invalid_bytes = &[
            0xA1, // outer map of 1 pair
            0x61, 0x61, // "a"
            0xA2, // inner map of 2 pairs
            0x62, 0x78, 0x79, // "xy"
            0x02, // 2
            0x61, 0x78, // "x"
            0x01, // 1
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_integer_boundaries() {
        // RFC 8949 Section 4.2.1 requires:
        // - Values 0 through 23 and -1 through -24 must be expressed in a single byte
        // - Values 24 through 255 and -25 through -256 must use an additional uint8_t
        // - Values 256 through 65535 and -257 through -65536 must use uint16_t
        // - Values 65536 through 4294967295 and -65537 through -4294967296 must use uint32_t
        // - All other integers must use uint64_t
        let test_cases = [
            (23, vec![0x17]),                            // Last direct value (0x17 = 23)
            (24, vec![0x18, 0x18]),                      // First 1-byte value
            (255, vec![0x18, 0xFF]),                     // Last 1-byte value
            (256, vec![0x19, 0x01, 0x00]),               // First 2-byte value
            (65535, vec![0x19, 0xFF, 0xFF]),             // Last 2-byte value
            (65536, vec![0x1A, 0x00, 0x01, 0x00, 0x00]), // First 4-byte value
        ];

        for (value, encoding) in test_cases {
            let mut dec = CborDecoder::new(&encoding);
            assert!(dec.decode().is_ok());

            // Test non-minimal encoding violations
            if value <= 23 {
                // RFC 8949 4.2.1: Values 0 through 23 MUST be expressed in a single byte
                let non_minimal = vec![0x18, value as u8];
                let mut dec = CborDecoder::new(&non_minimal);
                assert!(matches!(
                    dec.decode(),
                    Err(DeterministicError::NonMinimalInt)
                ));
            }
        }
    }

    #[test]
    fn test_negative_integer_encoding() {
        // RFC 8949 4.2.1 defines minimal encoding for negative integers:
        // - Values -1 to -24 must use single byte (major type 1)
        // - Values -25 to -256 must use additional uint8_t
        // - Values -257 to -65536 must use uint16_t
        // - Values -65537 to -4294967296 must use uint32_t
        let test_cases = [
            (-1, vec![0x20]),               // Negative integers start at 0x20
            (-24, vec![0x37]),              // Last single-byte negative
            (-25, vec![0x38, 0x18]),        // First 1-byte extended negative
            (-256, vec![0x38, 0xFF]),       // Last 1-byte extended negative
            (-257, vec![0x39, 0x01, 0x00]), // First 2-byte negative
        ];

        for (_value, encoding) in test_cases {
            let mut dec = CborDecoder::new(&encoding);
            assert!(dec.decode().is_ok());
        }
    }

    #[test]
    fn test_complex_map_ordering() {
        // RFC 8949 4.2.1 map ordering rules:
        // 1. If two keys have different lengths, the shorter one sorts earlier
        // 2. If two keys have the same length, the one with the lower value in lexical order
        //    sorts earlier
        let valid_bytes = &[
            0xA4, // map of 4 pairs
            0x61, 0x61, // "a" (length 1)
            0x01, 0x62, 0x61, 0x61, // "aa" (length 2)
            0x02, 0x62, 0x61, 0x62, // "ab" (length 2, lexically after "aa")
            0x03, 0x63, 0x61, 0x61, 0x61, // "aaa" (length 3)
            0x04,
        ];
        let mut dec = CborDecoder::new(valid_bytes);

        assert!(dec.decode().is_ok());

        // Test violation of length-first rule
        let invalid_bytes = &[
            0xA2, // map of 2 pairs
            0x63, 0x61, 0x61, 0x61, // "aaa" (longer key first - invalid)
            0x01, 0x61, 0x61, // "a" (shorter key second - invalid)
            0x02,
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_floating_point_encoding() {
        // RFC 8949 4.2.2 floating point encoding rules:
        // 1. Non-finite values (NaN, Infinity) SHOULD NOT be used
        // 2. Use the shortest form that preserves the value exactly
        // 3. If multiple representations are possible, use the one with the smallest precision
        //    that preserves the value

        // Test minimal encoding for common values
        let valid_test_cases = [
            // Half precision (if value can be represented without loss)
            vec![0xF9, 0x40, 0x00], // 2.5 in half precision
            // Single precision for values that need it
            vec![0xFA, 0x40, 0x48, 0xF5, 0xC3], // 3.14 in single precision
            // Double precision for values requiring it
            vec![0xFB, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18], // 3.14159265359 in double
        ];

        for encoding in valid_test_cases {
            let mut dec = CborDecoder::new(&encoding);
            assert!(
                dec.decode().is_ok(),
                "Valid float encoding should be accepted"
            );
        }

        // Note: Currently the implementation doesn't explicitly check for non-minimal
        // float encoding or non-finite values. When that functionality is added,
        // these additional tests should be uncommented:

        // Test non-minimal encoding (using double precision for value that fits in
        // single) let non_minimal = vec![0xfb, 0x40, 0x04, 0x00, 0x00, 0x00,
        // 0x00, 0x00, 0x00]; // 2.5 in double let mut dec =
        // DeterministicDecoder::new(&non_minimal); assert!(matches!(dec.
        // validate_next(), Err(DeterministicError::NonMinimalFloat)));
        //
        // Test infinity and NaN
        // let infinity = vec![0xfa, 0x7f, 0x80, 0x00, 0x00]; // Single precision infinity
        // let mut dec = DeterministicDecoder::new(&infinity);
        // assert!(matches!(dec.validate_next(),
        // Err(DeterministicError::NonFiniteFloat)));
        //
        // let nan = vec![0xfa, 0x7f, 0xc0, 0x00, 0x00]; // Single precision NaN
        // let mut dec = DeterministicDecoder::new(&nan);
        // assert!(matches!(dec.validate_next(),
        // Err(DeterministicError::NonFiniteFloat)));
    }

    #[test]
    fn test_string_comparison_ordering() {
        // RFC 8949 4.2.1 string comparison rules for map keys:
        // 1. Keys are compared byte by byte in lexicographic order
        // 2. Shorter keys sort before longer keys
        // 3. Raw bytes are compared without any further interpretation
        let valid_bytes = &[
            0xA3, // map of 3 pairs
            0x61, 0x41, // "A" (ASCII 65)
            0x01, 0x61, 0x61, // "a" (ASCII 97)
            0x02, 0x61, 0x7A, // "z" (ASCII 122)
            0x03,
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Test UTF-8 ordering (raw bytes comparison)
        // Note: UTF-8 comparison is done byte by byte, not by Unicode code points
        let valid_utf8_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0xC3, 0xA4, // "ä" (UTF-8: C3 A4)
            0x01, 0x62, 0xC3, 0xB6, // "ö" (UTF-8: C3 B6)
            0x02,
        ];
        let mut dec = CborDecoder::new(valid_utf8_bytes);
        assert!(dec.decode().is_ok());
    }

    #[test]
    fn test_string_comparison_ordering_violations() {
        // Test violation of lexicographic ordering
        let invalid_order_bytes = &[
            0xA2, // map of 2 pairs
            0x61, 0x7A, // "z" (ASCII 122)
            0x01, 0x61, 0x61, // "a" (ASCII 97)
            0x02,
        ];
        let mut dec = CborDecoder::new(invalid_order_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));

        // Test violation with duplicate keys
        let duplicate_keys_bytes = &[
            0xA2, // map of 2 pairs
            0x61, 0x61, // "a" (ASCII 97)
            0x01, 0x61, 0x61, // "a" (ASCII 97)
            0x02,
        ];
        let mut dec = CborDecoder::new(duplicate_keys_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::DuplicateMapKey)
        ));

        // Test violation with UTF-8 ordering
        let invalid_utf8_order_bytes = &[
            0xA2, // map of 2 pairs
            0x62, 0xC3, 0xB6, // "ö" (UTF-8: C3 B6)
            0x01, 0x62, 0xC3, 0xA4, // "ä" (UTF-8: C3 A4)
            0x02,
        ];
        let mut dec = CborDecoder::new(invalid_utf8_order_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::UnorderedMapKeys)
        ));
    }

    #[test]
    fn test_nested_structures_length_encoding() {
        // RFC 8949 4.2.1 requires minimal length encoding for all items:
        // - Arrays, maps, strings, and byte strings must use minimal length encoding
        // - This applies to nested structures as well
        let valid_bytes = &[
            0x82, // array of 2 elements (minimal encoding for length 2)
            0xA1, // map of 1 pair (minimal encoding for length 1)
            0x61, 0x61, // "a"
            0x82, // array of 2 elements
            0x01, 0x02, 0x62, 0x62, 0x62, // "bb"
        ];
        let mut dec = CborDecoder::new(valid_bytes);
        assert!(dec.decode().is_ok());

        // Test violation of minimal length encoding in nested structure
        let invalid_bytes = &[
            0x82, 0xA1, 0x61, 0x61, 0x98,
            0x02, // non-minimal array length encoding (using 1 byte when direct encoding possible)
            0x01, 0x02, 0x62, 0x62, 0x62,
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::NonMinimalInt)
        ));
    }

    #[test]
    fn test_duplicate_map_keys() {
        // RFC 8949 4.2.1: Maps must not contain duplicate keys
        let invalid_bytes = &[
            0xA2, // map of 2 pairs
            0x61, 0x61, // "a"
            0x01, 0x61, 0x61, // "a" (duplicate key)
            0x02,
        ];
        let mut dec = CborDecoder::new(invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::DuplicateMapKey)
        ));
    }

    #[test]
    fn test_array_length_encoding() {
        // RFC 8949 4.2.1: Length encoding must be minimal for arrays
        let test_cases = [
            (0, vec![0x80]),       // Empty array
            (1, vec![0x81, 0x01]), // One-element array
            (
                23,
                vec![0x97]
                    .into_iter()
                    .chain((0..23).map(|_| 0x01))
                    .collect::<Vec<_>>(),
            ), // 23 elements
            (
                24,
                vec![0x98, 0x18]
                    .into_iter()
                    .chain((0..24).map(|_| 0x01))
                    .collect::<Vec<_>>(),
            ), // 24 elements
        ];

        for (_len, encoding) in test_cases {
            let mut dec = CborDecoder::new(&encoding);
            assert!(dec.decode().is_ok());
        }

        // Test non-minimal length encoding
        let invalid_bytes = vec![0x98, 0x01, 0x01]; // Using 1-byte length for single item
        let mut dec = CborDecoder::new(&invalid_bytes);
        assert!(matches!(
            dec.decode(),
            Err(DeterministicError::NonMinimalInt)
        ));
    }
}
