//! A Catalyst generalized vote transaction object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/gen_vote_tx/)

mod builder;
mod event_map;
mod tx_body;
mod vote;

pub use builder::GeneralizedTxBuilder;
use coset::CborSerializable;
pub use event_map::{EventKey, EventMap};
use minicbor::{Decode, Decoder, Encode, Encoder};
pub use tx_body::{TxBody, VoterData};
pub use vote::{Choice, Proof, PropId, Vote};

use crate::Cbor;

/// A generalized tx struct.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneralizedTx<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    /// `tx-body` field
    tx_body: TxBody<ChoiceT, ProofT, PropIdT, VoterDataT>,
    /// `signature` field
    signature: coset::CoseSign,
}

/// `GeneralizedTx` array struct length
const GENERALIZED_TX_LEN: u64 = 2;

impl<ChoiceT, ProofT, PropIdT, VoterDataT> Decode<'_, ()>
    for GeneralizedTx<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    fn decode(
        d: &mut Decoder<'_>,
        (): &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
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

            if sign.protected.header != cose_protected_header() {
                return Err(minicbor::decode::Error::message(
                    "invalid `signature` COSE_Sign protected header",
                ));
            }

            sign
        };

        Ok(Self { tx_body, signature })
    }
}

impl<ChoiceT, ProofT, PropIdT, VoterDataT> Encode<()>
    for GeneralizedTx<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut Encoder<W>,
        (): &mut (),
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

/// Returns the COSE protected header for `GeneralizedTx` signature.
fn cose_protected_header() -> coset::Header {
    coset::HeaderBuilder::new()
        .content_format(coset::iana::CoapContentFormat::Cbor)
        .build()
}

#[allow(clippy::explicit_deref_methods)]
#[cfg(test)]
mod tests {
    use proptest::{prelude::any_with, sample::size_range};
    use proptest_derive::Arbitrary;
    use test_strategy::proptest;

    use super::*;
    use crate::{encoded_cbor::EncodedCbor, uuid::Uuid};

    type ChoiceT = Vec<u8>;
    type ProofT = Vec<u8>;
    type PropIdT = Vec<u8>;
    type VoterDataT = Vec<u8>;

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
                    choices: choices.into_iter().map(EncodedCbor).collect(),
                    proof: EncodedCbor(proof),
                    prop_id: EncodedCbor(prop_id),
                }
            })
            .collect();
        let tx_body = TxBody {
            vote_type: Uuid(vote_type),
            event: EventMap(event),
            votes,
            voter_data: EncodedCbor(voter_data),
        };
        let signature = coset::CoseSignBuilder::new()
            .protected(cose_protected_header())
            .build();

        let generalized_tx = GeneralizedTx { tx_body, signature };
        let bytes = generalized_tx.to_bytes().unwrap();
        let decoded = GeneralizedTx::from_bytes(&bytes).unwrap();
        assert_eq!(generalized_tx, decoded);
    }

    #[proptest]
    fn generalized_tx_with_empty_votes_from_bytes_to_bytes_test(
        vote_type: Vec<u8>,
        event: Vec<(PropEventKey, u64)>,
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

        let empty_votes = Vec::<Vote<ChoiceT, ProofT, PropIdT>>::new();
        let tx_body = TxBody {
            vote_type: Uuid(vote_type.clone()),
            event: EventMap(event.clone()),
            votes: empty_votes,
            voter_data: EncodedCbor(voter_data.clone()),
        };
        let signature = coset::CoseSignBuilder::new()
            .protected(cose_protected_header())
            .build();

        let generalized_tx = GeneralizedTx { tx_body, signature };
        let bytes = generalized_tx.to_bytes().unwrap();
        assert!(GeneralizedTx::<ChoiceT, ProofT, PropIdT, VoterDataT>::from_bytes(&bytes).is_err());
    }

    #[proptest]
    fn generalized_tx_with_empty_choices_from_bytes_to_bytes_test(
        vote_type: Vec<u8>,
        votes: Vec<PropVote>,
        event: Vec<(PropEventKey, u64)>,
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

        let votes_with_empty_choices = votes
            .into_iter()
            .map(|(_, proof, prop_id)| {
                Vote {
                    choices: Vec::<EncodedCbor<ChoiceT>>::new(),
                    proof: EncodedCbor(proof),
                    prop_id: EncodedCbor(prop_id),
                }
            })
            .collect();
        let tx_body = TxBody {
            vote_type: Uuid(vote_type),
            event: EventMap(event),
            votes: votes_with_empty_choices,
            voter_data: EncodedCbor(voter_data),
        };
        let signature = coset::CoseSignBuilder::new()
            .protected(cose_protected_header())
            .build();

        let generalized_tx = GeneralizedTx { tx_body, signature };
        let bytes = generalized_tx.to_bytes().unwrap();
        assert!(GeneralizedTx::<ChoiceT, ProofT, PropIdT, VoterDataT>::from_bytes(&bytes).is_err());
    }

    #[proptest]
    fn generalized_tx_with_wrong_signature_from_bytes_to_bytes_test(
        vote_type: Vec<u8>,
        votes: Vec<PropVote>,
        event: Vec<(PropEventKey, u64)>,
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

        let votes = votes
            .into_iter()
            .map(|(choices, proof, prop_id)| {
                Vote {
                    choices: choices.into_iter().map(EncodedCbor).collect(),
                    proof: EncodedCbor(proof),
                    prop_id: EncodedCbor(prop_id),
                }
            })
            .collect();
        let tx_body = TxBody {
            vote_type: Uuid(vote_type),
            event: EventMap(event),
            votes,
            voter_data: EncodedCbor(voter_data),
        };
        let signature = coset::CoseSignBuilder::new().build();

        let generalized_tx = GeneralizedTx { tx_body, signature };
        let bytes = generalized_tx.to_bytes().unwrap();
        assert!(GeneralizedTx::<ChoiceT, ProofT, PropIdT, VoterDataT>::from_bytes(&bytes).is_err());
    }
}
