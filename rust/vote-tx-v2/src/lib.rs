//! A Catalyst vote transaction v1 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/)

use anyhow::anyhow;
use minicbor::{Decode, Decoder, Encode, Encoder};

mod decoding;

/// A geeneralised tx struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralisedTx {
    /// `tx-body`
    tx_body: TxBody,
}

/// A tx body struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxBody {
    /// `vote-type` field
    vote_type: Uuid,
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

#[allow(missing_docs, clippy::missing_docs_in_private_items)]
mod arbitrary_impl {
    use proptest::{
        prelude::{any, any_with, Arbitrary, BoxedStrategy, Strategy},
        sample::size_range,
    };

    use super::{Choice, GeneralisedTx, Proof, PropId, TxBody, Uuid, Vote, VoterData};

    impl Arbitrary for GeneralisedTx {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<TxBody>().prop_map(|tx_body| Self { tx_body }).boxed()
        }
    }

    impl Arbitrary for TxBody {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(Vec<u8>, Vec<Vote>, Vec<u8>)>()
                .prop_map(|(vote_type, votes, voters_data)| {
                    Self {
                        vote_type: Uuid(vote_type),
                        votes,
                        voter_data: VoterData(voters_data),
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for Vote {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any_with::<(Vec<Vec<u8>>, Vec<u8>, Vec<u8>)>((
                (size_range(1..10usize), Default::default()),
                Default::default(),
                Default::default(),
            ))
            .prop_map(|(choices, proof, prop_id)| {
                Self {
                    choices: choices.into_iter().map(Choice).collect(),
                    proof: Proof(proof),
                    prop_id: PropId(prop_id),
                }
            })
            .boxed()
        }
    }
}
