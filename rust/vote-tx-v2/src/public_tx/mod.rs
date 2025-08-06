//! A Catalyst public vote transaction v2 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/#public-vote)

mod vote;

use std::ops::{Deref, DerefMut};

use minicbor::{Decode, Encode};
pub use vote::{Choice, Proof, PropId};

use crate::{gen_tx::GeneralizedTx, Cbor};

/// A public vote tx struct.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicTx<VoteDataT>(GeneralizedTx<Choice, Proof, PropId, VoteDataT>)
where VoteDataT: for<'a> Cbor<'a>;

impl<VoteDataT> Deref for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    type Target = GeneralizedTx<Choice, Proof, PropId, VoteDataT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<VoteDataT> DerefMut for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<VoteDataT> Decode<'_, ()> for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        (): &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let gen_tx = GeneralizedTx::decode(d, &mut ())?;
        Ok(Self(gen_tx))
    }
}

impl<VoteDataT> Encode<()> for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

#[cfg(test)]
mod tests {
    use proptest::sample::size_range;
    use test_strategy::proptest;

    use super::*;
    use crate::{encoded_cbor::EncodedCbor, gen_tx::GeneralizedTxBuilder, uuid::Uuid};

    #[proptest]
    fn public_tx_from_bytes_to_bytes_test(
        vote_type: Vec<u8>,
        voter_data: Vec<u8>,
        #[any(size_range(1..10_usize).lift())] choices: Vec<u64>,
        prop_id: Vec<u8>,
    ) {
        let gen_tx_builder = GeneralizedTxBuilder::<Choice, Proof, PropId, _>::new(
            Uuid(vote_type),
            EncodedCbor(voter_data),
        );
        let choices = choices.into_iter().map(Choice).collect();
        let gen_tx = gen_tx_builder
            .with_vote(choices, Proof, Uuid(prop_id))
            .unwrap()
            .build()
            .unwrap();
        let public_tx = PublicTx(gen_tx);

        let bytes = public_tx.to_bytes().unwrap();
        let decoded = PublicTx::from_bytes(&bytes).unwrap();
        assert_eq!(public_tx, decoded);
    }
}
