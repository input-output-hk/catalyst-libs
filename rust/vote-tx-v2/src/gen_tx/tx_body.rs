//! A generalized tx body struct.

use minicbor::{Decode, Decoder, Encode, Encoder};

use super::{EventMap, Vote};
use crate::{Cbor, encoded_cbor::EncodedCbor, uuid::Uuid};

/// `TxBody` array struct length
const TX_BODY_LEN: u64 = 4;

/// A voter's data type.
pub type VoterData<T> = EncodedCbor<T>;

/// A tx body struct.
#[derive(Debug, Clone, PartialEq)]
pub struct TxBody<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    /// `vote-type` field
    pub(super) vote_type: Uuid,
    /// `event` field
    pub(super) event: EventMap,
    /// `votes` field
    pub(super) votes: Vec<Vote<ChoiceT, ProofT, PropIdT>>,
    /// `voter-data` field
    pub(super) voter_data: VoterData<VoterDataT>,
}

impl<ChoiceT, ProofT, PropIdT, VoterDataT> Decode<'_, ()>
    for TxBody<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(TX_BODY_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {TX_BODY_LEN} entries"
            )));
        };

        let vote_type = Uuid::decode(d, &mut ())?;
        let event = EventMap::decode(d, &mut ())?;
        let votes = Vec::<Vote<_, _, _>>::decode(d, &mut ())?;
        if votes.is_empty() {
            return Err(minicbor::decode::Error::message(
                "votes array must has at least one entry",
            ));
        }
        let voter_data = VoterData::decode(d, &mut ())?;
        Ok(Self {
            vote_type,
            event,
            votes,
            voter_data,
        })
    }
}

impl<ChoiceT, ProofT, PropIdT, VoterDataT> Encode<()>
    for TxBody<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(TX_BODY_LEN)?;
        self.vote_type.encode(e, &mut ())?;
        self.event.encode(e, &mut ())?;
        self.votes.encode(e, &mut ())?;
        self.voter_data.encode(e, &mut ())?;
        Ok(())
    }
}
