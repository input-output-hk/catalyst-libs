//! A Catalyst vote transaction v1 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/)

use anyhow::anyhow;
use minicbor::{data::Int, Decode, Decoder, Encode, Encoder};

mod decoding;

/// A generalized tx struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralizedTx {
    /// `tx-body`
    tx_body: TxBody,
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

/// An `event-key` type defintion.
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

/// Cbor encodable and decodable type trait.
pub trait Cbor<'a>: Encode<()> + Decode<'a, ()> {
    /// Encodes to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut e = Encoder::new(&mut bytes);
        self.encode(&mut e, &mut ())
            .map_err(|e| anyhow!("Cannot encode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(bytes)
    }

    /// Decodes from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot decode
    fn from_bytes(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let mut decoder = Decoder::new(bytes);
        let res = Self::decode(&mut decoder, &mut ())
            .map_err(|e| anyhow!("Cannot decode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(res)
    }
}

impl<'a, T> Cbor<'a> for T where T: Encode<()> + Decode<'a, ()> {}
