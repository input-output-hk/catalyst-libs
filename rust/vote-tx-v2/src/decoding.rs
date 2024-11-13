//! CBOR encoding and decoding implementation.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/cddl/gen_vote_tx.cddl>

use minicbor::{
    data::{IanaTag, Tag},
    Decode, Decoder, Encode, Encoder,
};

use crate::{Choice, GeneralizedTx, Proof, PropId, TxBody, Uuid, Vote, VoterData};

/// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
const CBOR_UUID_TAG: u64 = 37;

/// `Vote` array struct length
const VOTE_LEN: u64 = 3;

/// `TxBody` array struct length
const TX_BODY_LEN: u64 = 3;

/// `GeneralizedTx` array struct length
const GENERALIZED_TX_LEN: u64 = 1;

impl Decode<'_, ()> for GeneralizedTx {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(GENERALIZED_TX_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {GENERALIZED_TX_LEN} entries"
            )));
        };

        let tx_body = TxBody::decode(d, &mut ())?;
        Ok(Self { tx_body })
    }
}

impl Encode<()> for GeneralizedTx {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(GENERALIZED_TX_LEN)?;
        self.tx_body.encode(e, &mut ())?;
        Ok(())
    }
}

impl Decode<'_, ()> for TxBody {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(TX_BODY_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {GENERALIZED_TX_LEN} entries"
            )));
        };

        let vote_type = Uuid::decode(d, &mut ())?;
        let votes = Vec::<Vote>::decode(d, &mut ())?;
        let voter_data = VoterData::decode(d, &mut ())?;
        Ok(Self {
            vote_type,
            votes,
            voter_data,
        })
    }
}

impl Encode<()> for TxBody {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(TX_BODY_LEN)?;
        self.vote_type.encode(e, &mut ())?;
        self.votes.encode(e, &mut ())?;
        self.voter_data.encode(e, &mut ())?;
        Ok(())
    }
}

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

impl Decode<'_, ()> for Uuid {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;
        if CBOR_UUID_TAG != tag.as_u64() {
            return Err(minicbor::decode::Error::message(format!(
                "tag value must be: {CBOR_UUID_TAG}, provided: {}",
                tag.as_u64(),
            )));
        }
        let choice = d.bytes()?.to_vec();
        Ok(Self(choice))
    }
}

impl Encode<()> for Uuid {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(Tag::new(CBOR_UUID_TAG))?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for Vote {
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(VOTE_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {VOTE_LEN} entries"
            )));
        };

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
        e.array(VOTE_LEN)?;
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
        Ok(Self(choice))
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
        Ok(Self(choice))
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
        Ok(Self(choice))
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
    use proptest::{prelude::any_with, sample::size_range};
    use test_strategy::proptest;

    use super::*;
    use crate::Cbor;

    type PropChoice = Vec<u8>;
    type PropVote = (Vec<PropChoice>, Vec<u8>, Vec<u8>);

    #[proptest]
    fn generalized_tx_from_bytes_to_bytes_test(
        vote_type: Vec<u8>,
        // generates a votes in range from 1 to 10, and choices in range from 1 to 10
        #[strategy(any_with::<Vec<PropVote>>((
            size_range(1..10usize),
            (
                (size_range(1..10usize), Default::default()),
                Default::default(),
                Default::default(),
            ),
        )))]
        votes: Vec<PropVote>,
        voter_data: Vec<u8>,
    ) {
        let generalized_tx = GeneralizedTx {
            tx_body: TxBody {
                vote_type: Uuid(vote_type),
                votes: votes
                    .into_iter()
                    .map(|(choices, proof, prop_id)| {
                        Vote {
                            choices: choices.into_iter().map(Choice).collect(),
                            proof: Proof(proof),
                            prop_id: PropId(prop_id),
                        }
                    })
                    .collect(),
                voter_data: VoterData(voter_data),
            },
        };

        let bytes = generalized_tx.to_bytes().unwrap();
        let decoded = GeneralizedTx::from_bytes(&bytes).unwrap();
        assert_eq!(generalized_tx, decoded);
    }
}
