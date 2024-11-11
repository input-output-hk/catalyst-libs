//! CBOR encoding and decoding implementation.

use anyhow::anyhow;
use minicbor::{data::IanaTag, Decode, Decoder, Encode, Encoder};

use crate::{Choice, Proof, PropId, Vote};

impl Vote {
    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote`
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut e = Encoder::new(&mut bytes);
        self.encode(&mut e, &mut ())
            .map_err(|e| anyhow!("Cannot encode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(bytes)
    }

    /// Decodes `Vote` from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot decode `Vote`
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let mut decoder = Decoder::new(bytes);
        let res = Vote::decode(&mut decoder, &mut ())
            .map_err(|e| anyhow!("Cannot decode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(res)
    }
}

impl Decode<'_, ()> for Vote {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        d.array()?;

        let choices = Vec::<Choice>::decode(d, &mut ())?;
        if choices.is_empty() {
            return Err(minicbor::decode::Error::message(
                "choices array must has at least one entry",
            ));
        }
        let proof = Proof::decode(d, &mut ())?;
        let prop_id = PropId::decode(d, &mut ())?;

        Ok(Self {
            choices,
            proof,
            prop_id,
        })
    }
}

impl Encode<()> for Vote {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(3)?;

        self.choices.encode(e, &mut ())?;
        self.proof.encode(e, &mut ())?;
        self.prop_id.encode(e, &mut ())?;
        Ok(())
    }
}

impl Decode<'_, ()> for Choice {
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
        Ok(Choice(choice))
    }
}

impl Encode<()> for Choice {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for Proof {
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
        Ok(Proof(choice))
    }
}

impl Encode<()> for Proof {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for PropId {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;
        let expected_tag = IanaTag::Cbor.tag();
        if expected_tag != tag {
            return Err(minicbor::decode::Error::message(format!(
                "tag value must be: {}, provided: {}",
                expected_tag.as_u64(),
                tag.as_u64(),
            )));
        }
        let choice = d.bytes()?.to_vec();
        Ok(PropId(choice))
    }
}

impl Encode<()> for PropId {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::ProptestConfig;
    use test_strategy::proptest;

    use super::Vote;

    #[proptest(ProptestConfig::with_cases(0))]
    fn vote_from_bytes_to_bytes_test(vote: Vote) {
        let bytes = vote.to_bytes().unwrap();
        let decoded = Vote::from_bytes(&bytes).unwrap();
        assert_eq!(vote, decoded);
    }
}
