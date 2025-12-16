//! D-Rep Encryption Key

use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

/// Placeholder of D-Rep Encryption Key.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct DrepEncryptionKey;

impl Encode<()> for DrepEncryptionKey {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.undefined()?;
        Ok(())
    }
}

impl Decode<'_, ()> for DrepEncryptionKey {
    fn decode(
        d: &mut Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        d.undefined()?;
        Ok(Self)
    }
}
