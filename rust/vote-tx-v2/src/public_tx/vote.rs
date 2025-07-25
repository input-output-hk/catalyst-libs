//! A public vote tx vote objects.

use minicbor::{Decode, Encode};

use crate::uuid::Uuid;

/// A public voting choice struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice(pub u64);

/// A public voting proof struct, CBOR `undefined`.
#[derive(Debug, Clone, PartialEq)]
pub struct Proof;

/// A public voting proposal id struct.
pub type PropId = Uuid;

impl Decode<'_, ()> for Choice {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        (): &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let choice = d.u64()?;
        Ok(Self(choice))
    }
}

impl Encode<()> for Choice {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, &mut ())
    }
}

impl Decode<'_, ()> for Proof {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        (): &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        d.undefined()?;
        Ok(Self)
    }
}

impl Encode<()> for Proof {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.undefined()?;
        Ok(())
    }
}
