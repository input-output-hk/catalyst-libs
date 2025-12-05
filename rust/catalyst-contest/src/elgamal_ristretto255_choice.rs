//! An elgamal encrypted ciphertext.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// An elgamal encrypted ciphertext `(c1, c2)`.
///
/// The CDDL schema:
/// ```cddl
/// elgamal-ristretto255-encrypted-choice = [
///     c1: elgamal-ristretto255-group-element
///     c2: elgamal-ristretto255-group-element
/// ]
///
/// elgamal-ristretto255-group-element = bytes .size 32
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ElgamalRistretto255Choice {
    /// An individual Elgamal group element that composes the elgamal cipher text.
    pub c1: [u8; 32],
    /// An individual Elgamal group element that composes the elgamal cipher text.
    pub c2: [u8; 32],
}

impl Decode<'_, ()> for ElgamalRistretto255Choice {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "elgamal ristretto255 choice")?;
        if len != 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected elgamal ristretto255 choice array length: {len}, expected 2"
            )));
        }
        let c1 = <[u8; 32]>::decode(d, ctx)?;
        let c2 = <[u8; 32]>::decode(d, ctx)?;
        Ok(ElgamalRistretto255Choice { c1, c2 })
    }
}

impl Encode<()> for ElgamalRistretto255Choice {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        self.c1.encode(e, ctx)?;
        self.c2.encode(e, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = ElgamalRistretto255Choice {
            c1: bytes,
            c2: bytes,
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            ElgamalRistretto255Choice::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
