//! A public vote tx choice struct.

use minicbor::{Decode, Encode};

/// A public voting choice struct.
pub struct Choice(u64);

impl Decode<'_, ()> for Choice {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let choice = d.u64()?;
        Ok(Self(choice))
    }
}

impl Encode<()> for Choice {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, &mut ())
    }
}
