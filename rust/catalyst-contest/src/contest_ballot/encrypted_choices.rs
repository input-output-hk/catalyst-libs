//! Encrypted voter choices.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::contest_ballot::encrypted_block::EncryptedBlock;

/// A CBOR version of the `EncryptedChoices`.
const VERSION: u64 = 0;

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
        if version != VERSION {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected EncryptedChoices version value: {version}, expected {VERSION}"
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
        VERSION.encode(e, ctx)?;
        for block in &self.0 {
            block.encode(e, ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::property_test;

    use super::*;

    #[property_test]
    fn roundtrip(block: EncryptedBlock) {
        let original = EncryptedChoices(vec![block]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedChoices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
