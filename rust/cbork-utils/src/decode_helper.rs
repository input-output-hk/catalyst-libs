//! CBOR decoding helper functions.

use minicbor::{data::Tag, decode, Decoder};

/// Generic helper function for decoding different types.
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_helper<'a, T, C>(
    d: &mut Decoder<'a>, from: &str, context: &mut C,
) -> Result<T, decode::Error>
where T: minicbor::Decode<'a, C> {
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
pub fn decode_bytes(d: &mut Decoder, from: &str) -> Result<Vec<u8>, decode::Error> {
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
pub fn decode_array_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
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
pub fn decode_map_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
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
pub fn decode_tag(d: &mut Decoder, from: &str) -> Result<Tag, decode::Error> {
    d.tag()
        .map_err(|e| decode::Error::message(format!("Failed to decode tag in {from}: {e}")))
}

/// Decode any in CDDL, only support basic datatype
///
/// # Errors
///
/// Error if the decoding fails.
pub fn decode_any(d: &mut Decoder, from: &str) -> Result<Vec<u8>, decode::Error> {
    match d.datatype()? {
        minicbor::data::Type::String => {
            match decode_helper::<String, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.as_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::U8 => {
            match decode_helper::<u8, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::U16 => {
            match decode_helper::<u16, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::U32 => {
            match decode_helper::<u32, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::U64 => {
            match decode_helper::<u64, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::I8 => {
            match decode_helper::<i8, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::I16 => {
            match decode_helper::<i16, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::I32 => {
            match decode_helper::<i32, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::I64 => {
            match decode_helper::<i64, _>(d, &format!("{from} Any"), &mut ()) {
                Ok(i) => Ok(i.to_be_bytes().to_vec()),
                Err(e) => Err(e),
            }
        },
        minicbor::data::Type::Bytes => Ok(decode_bytes(d, &format!("{from} Any"))?),
        minicbor::data::Type::Array => {
            Ok(decode_array_len(d, &format!("{from} Any"))?
                .to_be_bytes()
                .to_vec())
        },
        _ => {
            Err(decode::Error::message(format!(
                "{from} Any, Data type not supported"
            )))
        },
    }
}

#[cfg(test)]
mod tests {

    use minicbor::Encoder;

    use super::*;

    #[test]
    fn test_decode_any_bytes() {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.bytes(&[1, 2, 3, 4]).expect("Error encoding bytes");

        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test").expect("Error decoding bytes");
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_decode_any_string() {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.str("hello").expect("Error encoding string");

        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test").expect("Error decoding string");
        assert_eq!(result, b"hello".to_vec());
    }

    #[test]
    fn test_decode_any_array() {
        // The array should contain a supported type
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.array(2).expect("Error encoding array");
        e.u8(1).expect("Error encoding u8");
        e.u8(2).expect("Error encoding u8");
        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test").expect("Error decoding array");
        // The decode of array is just a length of the array
        assert_eq!(
            u64::from_be_bytes(result.try_into().expect("Error converting bytes to u64")),
            2
        );
    }

    #[test]
    fn test_decode_any_u32() {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        let num: u32 = 123_456_789;
        e.u32(num).expect("Error encoding u32");

        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test").expect("Error decoding u32");
        assert_eq!(
            u32::from_be_bytes(result.try_into().expect("Error converting bytes to u32")),
            num
        );
    }

    #[test]
    fn test_decode_any_i32() {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        let num: i32 = -123_456_789;
        e.i32(num).expect("Error encoding i32");
        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test").expect("Error decoding i32");
        assert_eq!(
            i32::from_be_bytes(result.try_into().expect("Error converting bytes to i32")),
            num
        );
    }

    #[test]
    fn test_decode_any_unsupported_type() {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.null().expect("Error encoding null"); // Encode a null type which is unsupported

        let mut d = Decoder::new(&buf);
        let result = decode_any(&mut d, "test");
        // Should print out the error message with the location of the error
        assert!(result.is_err());
    }
}
