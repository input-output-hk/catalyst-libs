//! An AES-CTR encrypted data block.

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A length of the encrypted block byte array.
const ENCRYPTED_BLOCK_LEN: usize = 16;

/// An AES-CTR encrypted data block.
///
/// The CDDL schema:
/// ```cddl
/// aes-ctr-encrypted-block = bytes .size 16
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct EncryptedBlock(pub [u8; ENCRYPTED_BLOCK_LEN]);

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
    fn roundtrip(original: EncryptedBlock) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedBlock::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
