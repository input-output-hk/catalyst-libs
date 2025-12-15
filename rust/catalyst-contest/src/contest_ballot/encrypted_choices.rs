//! Encrypted voter choices.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A length of the encrypted block byte array.
const ENCRYPTED_BLOCK_LEN: usize = 16;

/// Encrypted voter choices.
///
/// The CDDL schema:
/// ```cddl
/// voter-choice = [ 0, aes-ctr-encrypted-choices ]
///
/// aes-ctr-encrypted-choices = +aes-ctr-encrypted-block
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncryptedChoices(pub Vec<EncryptedBlock>);

/// An AES-CTR encrypted data block.
///
/// The CDDL schema:
/// ```cddl
/// aes-ctr-encrypted-block = bytes .size 16
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct EncryptedBlock(pub [u8; ENCRYPTED_BLOCK_LEN]);

impl Decode<'_, ()> for EncryptedChoices {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len: usize = decode_array_len(d, "EncryptedChoices")?
            .try_into()
            .map_err(minicbor::decode::Error::message)?;
        if len < 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected EncryptedChoices array length: {len}, expected at least 2"
            )));
        }
        let version = u64::decode(d, ctx)?;
        if version != 0 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected EncryptedChoices version value: {version}, expected 0"
            )));
        }

        // This is allowed because of the `len < 2` check above.
        #[allow(clippy::arithmetic_side_effects)]
        let mut blocks = Vec::with_capacity(len - 1);
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
        e.array((self.0.len() as u64).checked_add(1).ok_or_else(|| {
            minicbor::encode::Error::message("EncryptedChoices length overflow")
        })?)?;
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
        <[u8; ENCRYPTED_BLOCK_LEN]>::decode(d, ctx).map(Self)
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
    use proptest::property_test;

    use super::*;

    #[property_test]
    fn encrypted_block_roundtrip(original: EncryptedBlock) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedBlock::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[property_test]
    fn encrypted_choices_roundtrip(block: EncryptedBlock) {
        let original = EncryptedChoices(vec![block]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedChoices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
