//! A public vote tx proof struct.

use minicbor::{Decode, Encode};

/// A public voting proof struct, CBOR `undefined`.
pub struct Proof;

impl Decode<'_, ()> for Proof {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        d.undefined()?;
        Ok(Self)
    }
}

impl Encode<()> for Proof {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.undefined()?;
        Ok(())
    }
}
