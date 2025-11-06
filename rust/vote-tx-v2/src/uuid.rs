//! A CBOR encoded/decoded UUID struct.

use minicbor::{Decode, Decoder, Encode, data::Tag};

/// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
const UUID_TAG: u64 = 37;

/// A UUID struct, CBOR tag 37.
#[derive(Debug, Clone, PartialEq)]
pub struct Uuid(pub Vec<u8>);

impl Decode<'_, ()> for Uuid {
    fn decode(
        d: &mut Decoder<'_>,
        (): &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;
        if UUID_TAG != tag.as_u64() {
            return Err(minicbor::decode::Error::message(format!(
                "tag value must be: {UUID_TAG}, provided: {}",
                tag.as_u64(),
            )));
        }
        let choice = d.bytes()?.to_vec();
        Ok(Self(choice))
    }
}

impl Encode<()> for Uuid {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(Tag::new(UUID_TAG))?;
        e.bytes(&self.0)?;
        Ok(())
    }
}
