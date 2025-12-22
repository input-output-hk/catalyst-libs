//! Clear Choice

use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

/// Type for the `clear-choice` field.
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ClearChoice(i64);

impl From<i64> for ClearChoice {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl Encode<()> for ClearChoice {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.i64(self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for ClearChoice {
    fn decode(
        d: &mut Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = d.i64()?;
        Ok(Self(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_clear_choice() {
        let test_values = [0i64, 1, -1, 42, -42, i64::MAX, i64::MIN];

        for value in test_values {
            let original = ClearChoice(value);

            let mut buffer = Vec::new();
            original
                .encode(&mut Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded = ClearChoice::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
            assert_eq!(original, decoded);
        }
    }
}
