//! A private vote tx choice struct.

use catalyst_voting::vote_protocol::voter::EncryptedChoice;
use minicbor::{Decode, Encode};

/// A private voting choice struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice(pub EncryptedChoice);

impl Decode<'_, ()> for Choice {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let bytes = d
            .bytes()?
            .try_into()
            .map_err(minicbor::decode::Error::message)?;

        let choice =
            EncryptedChoice::from_bytes(&bytes).map_err(minicbor::decode::Error::message)?;

        Ok(Self(choice))
    }
}

impl Encode<()> for Choice {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let bytes = self.0.to_bytes();
        e.bytes(&bytes)?;
        Ok(())
    }
}
