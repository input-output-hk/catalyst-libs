//! A public vote tx proposal id struct.

use minicbor::{Decode, Encode};

use crate::gen_tx::Uuid;

/// A public voting proposal id struct.
pub struct PropId(Uuid);

impl Decode<'_, ()> for PropId {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let prop_id = Uuid::decode(d, &mut ())?;
        Ok(Self(prop_id))
    }
}

impl Encode<()> for PropId {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, &mut ())
    }
}
