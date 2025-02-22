//! An encoded CBOR (tag 24) struct

use minicbor::{Decode, Decoder, Encode, data::Tag};

use crate::Cbor;

/// encoded-cbor CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
const ENCODED_CBOR_TAG: u64 = 24;

/// An encoded CBOR struct, CBOR tag 24.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedCbor<T>(pub T)
where T: for<'a> Cbor<'a>;

impl<T> Decode<'_, ()> for EncodedCbor<T>
where T: for<'a> Cbor<'a>
{
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;
        if ENCODED_CBOR_TAG != tag.as_u64() {
            return Err(minicbor::decode::Error::message(format!(
                "tag value must be: {ENCODED_CBOR_TAG}, provided: {}",
                tag.as_u64(),
            )));
        }
        let cbor_bytes = d.bytes()?.to_vec();
        let cbor = T::from_bytes(&cbor_bytes).map_err(minicbor::decode::Error::message)?;
        Ok(Self(cbor))
    }
}

impl<T> Encode<()> for EncodedCbor<T>
where T: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(Tag::new(ENCODED_CBOR_TAG))?;
        let cbor_bytes = self
            .0
            .to_bytes()
            .map_err(minicbor::encode::Error::message)?;
        e.bytes(&cbor_bytes)?;
        Ok(())
    }
}
