//! A Catalyst generalized vote transaction builder

use anyhow::ensure;

use super::{cose_protected_header, EventKey, EventMap, GeneralizedTx, TxBody, Vote, VoterData};
use crate::{encoded_cbor::EncodedCbor, uuid::Uuid, Cbor};

/// `GeneralizedTx` builder struct
#[allow(clippy::module_name_repetitions)]
pub struct GeneralizedTxBuilder<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
    VoterDataT: for<'a> Cbor<'a>,
{
    /// The `vote_type` field
    vote_type: Uuid,
    /// The `event` field
    event: EventMap,
    /// The `votes` field
    votes: Vec<Vote<ChoiceT, ProofT, PropIdT>>,
    /// The `voter_data` field
    voter_data: VoterData<VoterDataT>,
    /// The `signature` builder field
    sign_builder: coset::CoseSignBuilder,
}

impl<ChoiceT, ProofT, PropIdT, VoterDataT>
    GeneralizedTxBuilder<ChoiceT, ProofT, PropIdT, VoterDataT>
where
    ChoiceT: for<'a> Cbor<'a> + Clone,
    ProofT: for<'a> Cbor<'a> + Clone,
    PropIdT: for<'a> Cbor<'a> + Clone,
    VoterDataT: for<'a> Cbor<'a> + Clone,
{
    /// Creates a new `GeneralizedTxBuilder` struct
    #[must_use]
    pub fn new(
        vote_type: Uuid,
        voter_data: VoterData<VoterDataT>,
    ) -> Self {
        let event = EventMap::default();
        let votes = Vec::default();
        let sign_builder = coset::CoseSignBuilder::new().protected(cose_protected_header());
        Self {
            vote_type,
            event,
            votes,
            voter_data,
            sign_builder,
        }
    }

    /// Adds an `EventMap` entry to the `event` field.
    ///
    /// # Errors
    pub fn with_event<ValueT>(
        mut self,
        key: EventKey,
        value: ValueT,
    ) -> anyhow::Result<Self>
    where
        ValueT: for<'a> Cbor<'a> + Clone,
    {
        let value = value.to_bytes()?;
        self.event.0.push((key, value));
        Ok(self)
    }

    /// Adds a `Vote` entry to the `votes` field.
    ///
    /// # Errors
    ///   - `choices` array must has at least one entry-
    pub fn with_vote(
        mut self,
        choices: Vec<ChoiceT>,
        proof: ProofT,
        prop_id: PropIdT,
    ) -> anyhow::Result<Self> {
        ensure!(
            !choices.is_empty(),
            "`choices` array must has at least one entry"
        );
        self.votes.push(Vote {
            choices: choices.into_iter().map(EncodedCbor).collect(),
            proof: EncodedCbor(proof),
            prop_id: EncodedCbor(prop_id),
        });
        Ok(self)
    }

    /// Builds a new `GeneralizedTx` object.
    ///
    /// # Errors
    ///  - `votes` array must has at least one entry
    pub fn build(self) -> anyhow::Result<GeneralizedTx<ChoiceT, ProofT, PropIdT, VoterDataT>> {
        ensure!(
            !self.votes.is_empty(),
            "`votes` array must has at least one entry"
        );

        let tx_body = TxBody {
            vote_type: self.vote_type,
            event: self.event,
            votes: self.votes,
            voter_data: self.voter_data,
        };
        let signature = self.sign_builder.build();
        Ok(GeneralizedTx { tx_body, signature })
    }
}
