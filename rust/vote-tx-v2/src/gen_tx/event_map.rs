//! A generalised tx event map struct.

use minicbor::{data::Int, Decode, Decoder, Encode, Encoder};

/// A CBOR map
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventMap(pub(super) Vec<(EventKey, Vec<u8>)>);

/// An `event-key` type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKey {
    /// CBOR `int` type
    Int(Int),
    /// CBOR `text` type
    Text(String),
}

impl Decode<'_, ()> for EventMap {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(len) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "must be a defined sized map",
            ));
        };

        let map = (0..len)
            .map(|_| {
                let key = EventKey::decode(d, &mut ())?;

                let value = read_cbor_bytes(d).map_err(|_| {
                    minicbor::decode::Error::message("missing event map `value` field")
                })?;
                Ok((key, value))
            })
            .collect::<Result<_, _>>()?;

        Ok(EventMap(map))
    }
}

impl Encode<()> for EventMap {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;

        for (key, value) in &self.0 {
            key.encode(e, &mut ())?;

            e.writer_mut()
                .write_all(value)
                .map_err(minicbor::encode::Error::write)?;
        }

        Ok(())
    }
}

impl Decode<'_, ()> for EventKey {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let pos = d.position();
        // try to decode as int
        if let Ok(i) = d.int() {
            Ok(EventKey::Int(i))
        } else {
            // try to decode as text
            d.set_position(pos);
            let str = d.str()?;
            Ok(EventKey::Text(str.to_string()))
        }
    }
}

impl Encode<()> for EventKey {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            EventKey::Int(i) => e.int(*i)?,
            EventKey::Text(s) => e.str(s)?,
        };
        Ok(())
    }
}

/// Reads CBOR bytes from the decoder and returns them as bytes.
fn read_cbor_bytes(d: &mut Decoder<'_>) -> Result<Vec<u8>, minicbor::decode::Error> {
    let start = d.position();
    d.skip()?;
    let end = d.position();
    let bytes = d
        .input()
        .get(start..end)
        .ok_or(minicbor::decode::Error::end_of_input())?
        .to_vec();
    Ok(bytes)
}
