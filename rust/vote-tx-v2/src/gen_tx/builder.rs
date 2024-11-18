//! A Catalyst generalised vote transaction builder

#![allow(dead_code)]

use anyhow::ensure;

use super::{
    cose_protected_header, EncodedCbor, EventKey, EventMap, GeneralizedTx, TxBody, Uuid, Vote,
    VoterData,
};
use crate::Cbor;

/// `GeneralizedTx` builder struct
#[allow(clippy::module_name_repetitions)]
pub struct GeneralizedTxBuilder<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    ProopIdT: for<'a> Cbor<'a>,
{
    /// The `vote_type` field
    vote_type: Uuid,
    /// The `event` field
    event: EventMap,
    /// The `votes` field
    votes: Vec<Vote<ChoiceT, ProofT, ProopIdT>>,
    /// The `voter_data` field
    voter_data: VoterData,
    /// The `signature` builder field
    sign_builder: coset::CoseSignBuilder,
}

impl<ChoiceT, ProofT, ProopIdT> GeneralizedTxBuilder<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a> + Clone,
    ProofT: for<'a> Cbor<'a> + Clone,
    ProopIdT: for<'a> Cbor<'a> + Clone,
{
    /// Creates a new `GeneralizedTxBuilder` struct
    #[must_use]
    pub fn new(vote_type: Uuid, voter_data: VoterData) -> Self {
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
    pub fn with_event<ValueT>(mut self, key: EventKey, value: ValueT) -> anyhow::Result<Self>
    where ValueT: for<'a> Cbor<'a> + Clone {
        let value = value.to_bytes()?;
        self.event.0.push((key, value));
        Ok(self)
    }

    /// Adds a `Vote` entry to the `votes` field.
    ///
    /// # Errors
    ///   - `choices` array must has at least one entry-
    pub fn with_vote(
        mut self, choices: Vec<ChoiceT>, proof: ProofT, prop_id: ProopIdT,
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
    pub fn build(&self) -> anyhow::Result<GeneralizedTx<ChoiceT, ProofT, ProopIdT>> {
        ensure!(
            !self.votes.is_empty(),
            "`votes` array must has at least one entry"
        );

        let tx_body = TxBody {
            vote_type: self.vote_type.clone(),
            event: self.event.clone(),
            votes: self.votes.clone(),
            voter_data: self.voter_data.clone(),
        };
        let signature = coset::CoseSignBuilder::new()
            .protected(cose_protected_header())
            .build();
        Ok(GeneralizedTx { tx_body, signature })
    }
}
