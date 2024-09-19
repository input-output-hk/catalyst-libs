//! Helper functions for encoding CBOR data.

use minicbor::{
    data::Tag,
    encode::{self, Write},
    Encoder,
};

/// Helper function for encoding u8.
pub(crate) fn encode_u8<W: Write>(
    e: &mut Encoder<W>, from: &str, value: u8,
) -> Result<(), encode::Error<W::Error>> {
    e.u8(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode u8 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding u16.
#[allow(dead_code)]
pub(crate) fn encode_u16<W: Write>(
    e: &mut Encoder<W>, from: &str, value: u16,
) -> Result<(), encode::Error<W::Error>> {
    e.u16(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode u16 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding u32.
#[allow(dead_code)]
pub(crate) fn encode_u32<W: Write>(
    e: &mut Encoder<W>, from: &str, value: u32,
) -> Result<(), encode::Error<W::Error>> {
    e.u32(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode u32 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding u64.
pub(crate) fn encode_u64<W: Write>(
    e: &mut Encoder<W>, from: &str, value: u64,
) -> Result<(), encode::Error<W::Error>> {
    e.u64(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode u64 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding i8.
#[allow(dead_code)]
pub(crate) fn encode_i8<W: Write>(
    e: &mut Encoder<W>, from: &str, value: i8,
) -> Result<(), encode::Error<W::Error>> {
    e.i8(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode i8 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding i16.
pub(crate) fn encode_i16<W: Write>(
    e: &mut Encoder<W>, from: &str, value: i16,
) -> Result<(), encode::Error<W::Error>> {
    e.i16(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode i16 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding i32.
#[allow(dead_code)]
pub(crate) fn encode_i32<W: Write>(
    e: &mut Encoder<W>, from: &str, value: i32,
) -> Result<(), encode::Error<W::Error>> {
    e.i32(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode i32 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding i64.
pub(crate) fn encode_i64<W: Write>(
    e: &mut Encoder<W>, from: &str, value: i64,
) -> Result<(), encode::Error<W::Error>> {
    e.i64(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode i64 in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding tag.
#[allow(dead_code)]
pub(crate) fn encode_tag<W: Write>(
    e: &mut Encoder<W>, from: &str, tag: Tag,
) -> Result<(), encode::Error<W::Error>> {
    e.tag(tag).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode tag in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding bytes.
pub(crate) fn encode_bytes<W: Write>(
    e: &mut Encoder<W>, from: &str, value: &[u8],
) -> Result<(), encode::Error<W::Error>> {
    e.bytes(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode bytes in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding null.
pub(crate) fn encode_null<W: Write>(
    e: &mut Encoder<W>, from: &str,
) -> Result<(), encode::Error<W::Error>> {
    e.null().map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode null in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding boolean.
pub(crate) fn encode_bool<W: Write>(
    e: &mut Encoder<W>, from: &str, value: bool,
) -> Result<(), encode::Error<W::Error>> {
    e.bool(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode boolean in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding string.
pub(crate) fn encode_str<W: Write>(
    e: &mut Encoder<W>, from: &str, value: &str,
) -> Result<(), encode::Error<W::Error>> {
    e.str(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode string in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding array.
pub(crate) fn encode_array_len<W: Write>(
    e: &mut Encoder<W>, from: &str, len: u64,
) -> Result<(), encode::Error<W::Error>> {
    e.array(len).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode array in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding map.
#[allow(dead_code)]
pub(crate) fn encode_map_len<W: Write>(
    e: &mut Encoder<W>, from: &str, value: u64,
) -> Result<(), encode::Error<W::Error>> {
    e.map(value).map_err(|err| {
        encode::Error::with_message(err, &format!("Failed to encode map in {from}"))
    })?;
    Ok(())
}
