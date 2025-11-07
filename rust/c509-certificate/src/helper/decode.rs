//! Helper functions for decoding CBOR data.

use minicbor::{Decoder, decode};

/// Generic helper function for decoding different types.
pub(crate) fn decode_helper<'a, T, C>(
    d: &mut Decoder<'a>,
    from: &str,
    context: &mut C,
) -> Result<T, decode::Error>
where
    T: minicbor::Decode<'a, C>,
{
    T::decode(d, context).map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode {:?} in {from}: {e}",
            std::any::type_name::<T>()
        ))
    })
}

/// Helper function for decoding bytes.
pub(crate) fn decode_bytes(
    d: &mut Decoder,
    from: &str,
) -> Result<Vec<u8>, decode::Error> {
    d.bytes().map(<[u8]>::to_vec).map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode bytes in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding array.
pub(crate) fn decode_array_len(
    d: &mut Decoder,
    from: &str,
) -> Result<u64, decode::Error> {
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

/// Helper function for decoding null.
pub(crate) fn decode_null(
    d: &mut Decoder,
    from: &str,
) -> Result<(), decode::Error> {
    d.null().map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode null in {from}:
            {e}"
        ))
    })
}

/// Helper function for decoding datatype.
pub(crate) fn decode_datatype(
    d: &mut Decoder,
    from: &str,
) -> Result<minicbor::data::Type, decode::Error> {
    d.datatype().map_err(|e| {
        decode::Error::message(format!(
            "Failed to decode datatype in {from}:
            {e}"
        ))
    })
}
