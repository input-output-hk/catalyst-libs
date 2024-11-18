//! A generalised tx voter's data struct.

use minicbor::{data::IanaTag, Decode, Decoder, Encode};

/// A voter's data struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoterData(pub Vec<u8>);

impl Decode<'_, ()> for VoterData {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;
        let expected_tag = minicbor::data::IanaTag::Cbor.tag();
        if expected_tag != tag {
            return Err(minicbor::decode::Error::message(format!(
                "tag value must be: {}, provided: {}",
                expected_tag.as_u64(),
                tag.as_u64(),
            )));
        }
        let choice = d.bytes()?.to_vec();
        Ok(Self(choice))
    }
}

impl Encode<()> for VoterData {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        e.bytes(&self.0)?;
        Ok(())
    }
}
