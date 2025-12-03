//! A universal encrypted matrix proof.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A length of the underlying CBOR array.
const ARRAY_LEN: u64 = 2;

/// A universal encrypted matrix proof.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MatrixProof(pub u64);

impl Decode<'_, ()> for MatrixProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "matrix proof")?;
        if len != ARRAY_LEN {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected matrix proof array length {len}, expected {ARRAY_LEN}"
            )));
        }
        let val = u64::decode(d, ctx)?;
        d.undefined()?;
        Ok(Self(val))
    }
}

impl Encode<()> for MatrixProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(ARRAY_LEN)?;
        self.0.encode(e, ctx)?;
        e.undefined()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let original = MatrixProof(1);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = MatrixProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
