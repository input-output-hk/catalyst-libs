//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

#![allow(unused_variables, dead_code)]

mod decoding;

use crate::{
    vote_protocol::voter::{proof::VoterProof, EncryptedVote},
    PublicKey,
};

/// A v1 (Jörmungandr) transaction struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    /// Vote plan id
    vote_plan_id: [u8; 32],
    /// Proposal index
    proposal_index: u8,
    /// Vote
    vote: Vote,
    /// Public key
    public_key: PublicKey,
}

/// Vote struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    /// Public voting choice
    Public(u8),
    /// Private (encrypted) voting choice
    Private(EncryptedVote, VoterProof),
}