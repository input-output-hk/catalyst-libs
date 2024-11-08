//! A Catalyst vote transaction v1 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/)

mod decoding;

/// A generalised vote transaction struct.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct GeneralisedTx {
    /// `tx-body` field
    tx_body: GeneralisedTxBody,
}

/// A generalised vote transaction body struct.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct GeneralisedTxBody {
    /// `vote-type` field
    vote_type: Vec<u8>,
    /// `event` field
    event: Vec<(ciborium::Value, ciborium::Value)>,
    /// `votes` field
    votes: Vec<Vote>,
    /// `voters-data` field
    voters_data: Vec<u8>,
}

/// A vote struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Vote {
    /// `choices` field
    choices: Vec<Vec<u8>>,
    /// `proof` field
    proof: Vec<u8>,
    /// `prop-id` field
    prop_id: Vec<u8>,
}
