//! A Catalyst generalised vote transaction object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/gen_vote_tx/)

// cspell: words Coap

mod decoding;
mod event_map;
mod tx_body;
mod vote;
mod voter_data;

pub use event_map::{EventKey, EventMap};
pub use tx_body::TxBody;
pub use vote::{Choice, Proof, PropId, Vote};
pub use voter_data::VoterData;

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

/// A UUID struct, CBOR tag 37.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(Vec<u8>);

/// An encoded CBOR struct, CBOR tag 24.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedCbor<T>(T)
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
