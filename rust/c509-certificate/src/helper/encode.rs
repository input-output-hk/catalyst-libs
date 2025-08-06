//! Helper functions for encoding CBOR data.

use minicbor::{
    encode::{self, Write},
    Encoder,
};

/// Generic helper function for encoding different types.
pub(crate) fn encode_helper<W: Write, C, T>(
    e: &mut Encoder<W>,
    from: &str,
    ctx: &mut C,
    value: &T,
) -> Result<(), encode::Error<W::Error>>
where
    T: minicbor::Encode<C>,
{
    T::encode(value, e, ctx).map_err(|err| {
        encode::Error::with_message(
            err,
            format!(
                "Failed to encode {:?} in {from}",
                std::any::type_name::<T>()
            ),
        )
    })?;

    Ok(())
}

/// Helper function for encoding bytes.
pub(crate) fn encode_bytes<W: Write>(
    e: &mut Encoder<W>,
    from: &str,
    value: &[u8],
) -> Result<(), encode::Error<W::Error>> {
    e.bytes(value).map_err(|err| {
        encode::Error::with_message(err, format!("Failed to encode bytes in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding null.
pub(crate) fn encode_null<W: Write>(
    e: &mut Encoder<W>,
    from: &str,
) -> Result<(), encode::Error<W::Error>> {
    e.null().map_err(|err| {
        encode::Error::with_message(err, format!("Failed to encode null in {from}"))
    })?;
    Ok(())
}

/// Helper function for encoding array.
pub(crate) fn encode_array_len<W: Write>(
    e: &mut Encoder<W>,
    from: &str,
    len: u64,
) -> Result<(), encode::Error<W::Error>> {
    e.array(len).map_err(|err| {
        encode::Error::with_message(err, format!("Failed to encode array in {from}"))
    })?;
    Ok(())
}
