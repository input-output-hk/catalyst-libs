//! Elgamal objects decoding implementation

use anyhow::anyhow;
use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use super::{Ciphertext, GroupElement};

impl Ciphertext {
    /// `Ciphertext` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE * 2;

    /// Convert this `Ciphertext` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; Self::BYTES_SIZE];
        res[0..32].copy_from_slice(&self.0.to_bytes());
        res[32..64].copy_from_slice(&self.1.to_bytes());
        res
    }

    /// Attempt to construct a `Ciphertext` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode group element field.
    ///
    /// # Panics
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(Self(
            GroupElement::from_bytes(bytes[0..32].try_into().unwrap())
                .map_err(|_| anyhow!("Cannot decode first group element field."))?,
            GroupElement::from_bytes(bytes[32..64].try_into().unwrap())
                .map_err(|_| anyhow!("Cannot decode second group element field."))?,
        ))
    }
}

impl Encode<()> for Ciphertext {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        self.0.encode(e, ctx)?;
        self.1.encode(e, ctx)
    }
}

impl Decode<'_, ()> for Ciphertext {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "Ciphertext")?;
        if len != 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected Ciphertext array length: {len}, expected 2"
            )));
        }
        let c1 = GroupElement::decode(d, ctx)?;
        let c2 = GroupElement::decode(d, ctx)?;
        Ok(Self(c1, c2))
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn ciphertext_to_bytes_from_bytes_test(c1: Ciphertext) {
        let bytes = c1.to_bytes();
        let c2 = Ciphertext::from_bytes(&bytes).unwrap();
        assert_eq!(c1, c2);
    }

    #[proptest]
    fn cbor_roundtrip(original: Ciphertext) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Ciphertext::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
