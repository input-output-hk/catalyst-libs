//! A generalised tx body struct.

use minicbor::{Decode, Decoder, Encode, Encoder};

use super::{EventMap, Uuid, Vote, VoterData};
use crate::Cbor;

/// `TxBody` array struct length
const TX_BODY_LEN: u64 = 4;

/// A tx body struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxBody<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    ProopIdT: for<'a> Cbor<'a>,
{
    /// `vote-type` field
    pub(super) vote_type: Uuid,
    /// `event` field
    pub(super) event: EventMap,
    /// `votes` field
    pub(super) votes: Vec<Vote<ChoiceT, ProofT, ProopIdT>>,
    /// `voter-data` field
    pub(super) voter_data: VoterData,
}

impl<ChoiceT, ProofT, PropIdT> Decode<'_, ()> for TxBody<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
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

impl<ChoiceT, ProofT, PropIdT> Encode<()> for TxBody<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
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
