//! CBOR encoding and decoding implementation.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/cddl/gen_vote_tx.cddl>

use coset::CborSerializable;
use minicbor::{
    data::{IanaTag, Tag},
    Decode, Decoder, Encode, Encoder,
};

use super::{Choice, GeneralizedTx, Proof, PropId, TxBody, Uuid};
use crate::Cbor;

/// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
const CBOR_UUID_TAG: u64 = 37;

/// `GeneralizedTx` array struct length
const GENERALIZED_TX_LEN: u64 = 2;

impl<ChoiceT, ProofT, ProopIdT> Decode<'_, ()> for GeneralizedTx<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    ProopIdT: for<'a> Cbor<'a>,
{
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(GENERALIZED_TX_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {GENERALIZED_TX_LEN} entries"
            )));
        };

        let tx_body = TxBody::decode(d, &mut ())?;

        let signature = {
            let sign_bytes = read_cbor_bytes(d)
                .map_err(|_| minicbor::decode::Error::message("missing `signature` field"))?;
            let mut sign = coset::CoseSign::from_slice(&sign_bytes).map_err(|_| {
                minicbor::decode::Error::message("`signature` must be COSE_Sign encoded object")
            })?;
            // We don't need to hold the original encoded data of the COSE protected header
            sign.protected.original_data = None;
            sign
        };

        Ok(Self { tx_body, signature })
    }
}

impl<ChoiceT, ProofT, PropIdT> Encode<()> for GeneralizedTx<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(GENERALIZED_TX_LEN)?;
        self.tx_body.encode(e, &mut ())?;

        let sign_bytes = self
            .signature
            .clone()
            .to_vec()
            .map_err(minicbor::encode::Error::message)?;
        e.writer_mut()
            .write_all(&sign_bytes)
            .map_err(minicbor::encode::Error::write)?;

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

impl<T> Decode<'_, ()> for Choice<T>
where T: for<'a> Cbor<'a>
{
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
        let choice_bytes = d.bytes()?.to_vec();
        let choice = T::from_bytes(&choice_bytes).map_err(minicbor::decode::Error::message)?;
        Ok(Self(choice))
    }
}

impl<T> Encode<()> for Choice<T>
where T: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        let choice_bytes = self
            .0
            .to_bytes()
            .map_err(minicbor::encode::Error::message)?;
        e.bytes(&choice_bytes)?;
        Ok(())
    }
}

impl<T> Decode<'_, ()> for Proof<T>
where T: for<'a> Cbor<'a>
{
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
        let proof_bytes = d.bytes()?.to_vec();
        let proof = T::from_bytes(&proof_bytes).map_err(minicbor::decode::Error::message)?;
        Ok(Self(proof))
    }
}

impl<T> Encode<()> for Proof<T>
where T: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        let proof_bytes = self
            .0
            .to_bytes()
            .map_err(minicbor::encode::Error::message)?;
        e.bytes(&proof_bytes)?;
        Ok(())
    }
}

impl<T> Decode<'_, ()> for PropId<T>
where T: for<'a> Cbor<'a>
{
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
        let prop_id_bytes = d.bytes()?.to_vec();
        let prop_id = T::from_bytes(&prop_id_bytes).map_err(minicbor::decode::Error::message)?;
        Ok(Self(prop_id))
    }
}

impl<T> Encode<()> for PropId<T>
where T: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(IanaTag::Cbor.tag())?;
        let prop_id_bytes = self
            .0
            .to_bytes()
            .map_err(minicbor::encode::Error::message)?;
        e.bytes(&prop_id_bytes)?;
        Ok(())
    }
}

/// Reads CBOR bytes from the decoder and returns them as bytes.
fn read_cbor_bytes(d: &mut Decoder<'_>) -> Result<Vec<u8>, minicbor::decode::Error> {
    let start = d.position();
    d.skip()?;
    let end = d.position();
    let bytes = d
        .input()
        .get(start..end)
        .ok_or(minicbor::decode::Error::end_of_input())?
        .to_vec();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::any_with, sample::size_range};
    use proptest_derive::Arbitrary;
    use test_strategy::proptest;

    use super::{
        super::{EventKey, EventMap, Vote, VoterData},
        *,
    };

    type ChoiceT = Vec<u8>;
    type ProofT = Vec<u8>;
    type PropIdT = Vec<u8>;

    type PropVote = (Vec<ChoiceT>, ProofT, PropIdT);

    #[derive(Debug, Arbitrary)]
    enum PropEventKey {
        Text(String),
        U64(u64),
        I64(i64),
    }

    impl From<PropEventKey> for EventKey {
        fn from(key: PropEventKey) -> Self {
            match key {
                PropEventKey::Text(text) => EventKey::Text(text),
                PropEventKey::U64(val) => EventKey::Int(val.into()),
                PropEventKey::I64(val) => EventKey::Int(val.into()),
            }
        }
    }

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
        event: Vec<(PropEventKey, u64)>,
        voter_data: Vec<u8>,
    ) {
        let event = event
            .into_iter()
            .map(|(key, val)| {
                let key = key.into();
                let value = val.to_bytes().unwrap();
                (key, value)
            })
            .collect();
        let votes = votes
            .into_iter()
            .map(|(choices, proof, prop_id)| {
                Vote {
                    choices: choices.into_iter().map(Choice).collect(),
                    proof: Proof(proof),
                    prop_id: PropId(prop_id),
                }
            })
            .collect();
        let tx_body = TxBody {
            vote_type: Uuid(vote_type),
            event: EventMap(event),
            votes,
            voter_data: VoterData(voter_data),
        };

        let generalized_tx = GeneralizedTx::<ChoiceT, ProofT, PropIdT>::new(tx_body);

        let bytes = generalized_tx.to_bytes().unwrap();
        let decoded = GeneralizedTx::<ChoiceT, ProofT, PropIdT>::from_bytes(&bytes).unwrap();
        assert_eq!(generalized_tx, decoded);
    }

    #[proptest]
    fn invalid_generalized_tx_from_bytes_to_bytes_test(
        vote_type: Vec<u8>, votes: Vec<PropVote>, event: Vec<(PropEventKey, u64)>,
        voter_data: Vec<u8>,
    ) {
        let event: Vec<_> = event
            .into_iter()
            .map(|(key, val)| {
                let key = key.into();
                let value = val.to_bytes().unwrap();
                (key, value)
            })
            .collect();
        // Empty votes case
        {
            let empty_votes = Vec::<Vote<ChoiceT, ProofT, PropIdT>>::new();
            let tx_body = TxBody {
                vote_type: Uuid(vote_type.clone()),
                event: EventMap(event.clone()),
                votes: empty_votes,
                voter_data: VoterData(voter_data.clone()),
            };

            let generalized_tx = GeneralizedTx::new(tx_body);
            let bytes = generalized_tx.to_bytes().unwrap();
            assert!(GeneralizedTx::<ChoiceT, ProofT, PropIdT>::from_bytes(&bytes).is_err());
        }

        // Empty choices case
        {
            let votes_with_empty_choices = votes
                .into_iter()
                .map(|(_, proof, prop_id)| {
                    Vote {
                        choices: Vec::<Choice<ChoiceT>>::new(),
                        proof: Proof(proof),
                        prop_id: PropId(prop_id),
                    }
                })
                .collect();
            let tx_body = TxBody {
                vote_type: Uuid(vote_type),
                event: EventMap(event),
                votes: votes_with_empty_choices,
                voter_data: VoterData(voter_data),
            };

            let generalized_tx = GeneralizedTx::new(tx_body);
            let bytes = generalized_tx.to_bytes().unwrap();
            assert!(GeneralizedTx::<ChoiceT, ProofT, PropIdT>::from_bytes(&bytes).is_err());
        }
    }
}
