//! A Catalyst generalised vote transaction object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/gen_vote_tx/)

// cspell: words Coap

mod decoding;
mod tx_body;
mod vote;

use minicbor::data::Int;
pub use tx_body::TxBody;
pub use vote::Vote;

use crate::Cbor;

/// A generalized tx struct.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneralizedTx<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    ProopIdT: for<'a> Cbor<'a>,
{
    /// `tx-body` field
    tx_body: TxBody<ChoiceT, ProofT, ProopIdT>,
    /// `signature` field
    signature: coset::CoseSign,
}

/// A CBOR map
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
pub struct Choice<T>(T)
where T: for<'a> Cbor<'a>;

/// A proof struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof<T>(T)
where T: for<'a> Cbor<'a>;

/// A prop id struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropId<T>(T)
where T: for<'a> Cbor<'a>;

impl<ChoiceT, ProofT, ProopIdT> GeneralizedTx<ChoiceT, ProofT, ProopIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    ProopIdT: for<'a> Cbor<'a>,
{
    /// Creates a new `GeneralizedTx` struct.
    #[must_use]
    pub fn new(tx_body: TxBody<ChoiceT, ProofT, ProopIdT>) -> Self {
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
