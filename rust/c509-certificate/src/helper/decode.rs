//! Helper functions for decoding CBOR data.

use minicbor::{data::Tag, decode, Decoder};

/// Helper function for decoding u8.
pub(crate) fn decode_u8(d: &mut Decoder, from: &str) -> Result<u8, decode::Error> {
    d.u8().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode u8 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding u16.
#[allow(dead_code)]
pub(crate) fn decode_u16(d: &mut Decoder, from: &str) -> Result<u16, decode::Error> {
    d.u16().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode u16 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding u32.
#[allow(dead_code)]
pub(crate) fn decode_u32(d: &mut Decoder, from: &str) -> Result<u32, decode::Error> {
    d.u32().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode u32 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding u64.
pub(crate) fn decode_u64(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
    d.u64().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode u64 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding i8.
#[allow(dead_code)]
pub(crate) fn decode_i8(d: &mut Decoder, from: &str) -> Result<i8, decode::Error> {
    d.i8().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode i8 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding i16.
pub(crate) fn decode_i16(d: &mut Decoder, from: &str) -> Result<i16, decode::Error> {
    d.i16().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode i16 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding i32.
#[allow(dead_code)]
pub(crate) fn decode_i32(d: &mut Decoder, from: &str) -> Result<i32, decode::Error> {
    d.i32().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode i32 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding i64.
pub(crate) fn decode_i64(d: &mut Decoder, from: &str) -> Result<i64, decode::Error> {
    d.i64().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode i64 in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding string.
pub(crate) fn decode_str(d: &mut Decoder, from: &str) -> Result<String, decode::Error> {
    d.str().map(std::borrow::ToOwned::to_owned).map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode string in
            {from}: {e}"
        ))
    })
}

/// Helper function for decoding bytes.
pub(crate) fn decode_bytes(d: &mut Decoder, from: &str) -> Result<Vec<u8>, decode::Error> {
    d.bytes().map(<[u8]>::to_vec).map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode bytes in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding array.
pub(crate) fn decode_array_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
    d.array()
        .map_err(|e| {
            decode::Error::message(&format!(
                "Failed to decode array in {from}:
            {e}"
            ))
        })?
        .ok_or(decode::Error::message(&format!(
            "Failed to decode array in {from}, unexpected indefinite length",
        )))
}

/// Helper function for decoding map.
#[allow(dead_code)]
pub(crate) fn decode_map_len(d: &mut Decoder, from: &str) -> Result<u64, decode::Error> {
    d.map()
        .map_err(|e| {
            decode::Error::message(&format!(
                "Failed to decode map in {from}:
            {e}"
            ))
        })?
        .ok_or(decode::Error::message(&format!(
            "Failed to decode map in {from}, unexpected indefinite length",
        )))
}

/// Helper function for decoding tag.
#[allow(dead_code)]
pub(crate) fn decode_tag(d: &mut Decoder, from: &str) -> Result<Tag, decode::Error> {
    d.tag().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode tag in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding null.
pub(crate) fn decode_null(d: &mut Decoder, from: &str) -> Result<(), decode::Error> {
    d.null().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode null in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding boolean.
pub(crate) fn decode_bool(d: &mut Decoder, from: &str) -> Result<bool, decode::Error> {
    d.bool().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode bool in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding datatype.
pub(crate) fn decode_datatype(
    d: &mut Decoder, from: &str,
) -> Result<minicbor::data::Type, decode::Error> {
    d.datatype().map_err(|e| {
        decode::Error::message(&format!(
            "Failed to decode datatype in {from}:
            {e}"
        ))
    })
}
