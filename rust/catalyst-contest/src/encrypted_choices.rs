//! Encrypted voter choices.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A length of the encrypted block array.
const ENCRYPTED_BLOCK_ARRAY_LEN: u64 = 16;

/// Encrypted voter choices.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncryptedChoices(pub Vec<EncryptedBlock>);

/// An AES-CTR encrypted data block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncryptedBlock(pub [u8; ENCRYPTED_BLOCK_ARRAY_LEN as usize]);

impl Decode<'_, ()> for EncryptedChoices {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "encrypted choices")?;
        if len < 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected encrypted choices array length: {len}, expected at least 2"
            )));
        }
        let val = u64::decode(d, ctx)?;
        if val != 0 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected encrypted choices array value: {val}, expected 0"
            )));
        }

        let mut blocks = Vec::with_capacity(len as usize - 1);
        for _ in 1..len {
            blocks.push(EncryptedBlock::decode(d, ctx)?);
        }

        Ok(Self(blocks))
    }
}

impl Encode<()> for EncryptedChoices {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(self.0.len() as u64 + 1)?;
        0.encode(e, ctx)?;
        for block in &self.0 {
            block.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for EncryptedBlock {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        <[u8; ENCRYPTED_BLOCK_ARRAY_LEN as usize]>::decode(d, ctx).map(EncryptedBlock)
    }
}

impl Encode<()> for EncryptedBlock {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_block_roundtrip() {
        let original = EncryptedBlock([2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedBlock::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn encrypted_choices_roundtrip() {
        let original = EncryptedChoices(vec![EncryptedBlock([
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ])]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedChoices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
