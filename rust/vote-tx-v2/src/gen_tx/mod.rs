//! A Catalyst generalised vote transaction object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/gen_vote_tx/)

// cspell: words Coap

mod decoding;

use minicbor::data::Int;

/// A generalized tx struct.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneralizedTx {
    /// `tx-body` field
    tx_body: TxBody,
    /// `signature` field
    signature: coset::CoseSign,
}

/// A tx body struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxBody {
    /// `vote-type` field
    vote_type: Uuid,
    /// `event` field
    event: EventMap,
    /// `votes` field
    votes: Vec<Vote>,
    /// `voter-data` field
    voter_data: VoterData,
}

/// A vote struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    /// `choices` field
    choices: Vec<Choice>,
    /// `proof` field
    proof: Proof,
    /// `prop-id` field
    prop_id: PropId,
}

/// A CBOR map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMap(Vec<(EventKey, Vec<u8>)>);

/// An `event-key` type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKey {
    /// CBOR `int` type
    Int(Int),
    /// CBOR `text` type
    Text(String),
}

/// A UUID struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(Vec<u8>);

/// A voter's data struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoterData(Vec<u8>);

/// A choice struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice(Vec<u8>);

/// A proof struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(Vec<u8>);

/// A prop id struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropId(Vec<u8>);

impl GeneralizedTx {
    /// Creates a new `GeneralizedTx` struct.
    #[must_use]
    pub fn new(tx_body: TxBody) -> Self {
        let signature = coset::CoseSignBuilder::new()
            .protected(Self::cose_protected_header())
            .build();
        Self { tx_body, signature }
    }

    /// Returns the COSE protected header.
    fn cose_protected_header() -> coset::Header {
        coset::HeaderBuilder::new()
            .content_format(coset::iana::CoapContentFormat::Cbor)
            .build()
    }
}
