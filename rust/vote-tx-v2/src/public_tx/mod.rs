//! A Catalyst public vote transaction v2 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/#public-vote)

mod choice;
mod proof;
mod prop_id;

use std::ops::{Deref, DerefMut};

pub use choice::Choice;
use minicbor::{Decode, Encode};
pub use proof::Proof;
pub use prop_id::PropId;

use crate::{gen_tx::GeneralizedTx, Cbor};

/// A public vote tx struct.
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
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let gen_tx = GeneralizedTx::decode(d, &mut ())?;
        Ok(Self(gen_tx))
    }
}

impl<VoteDataT> Encode<()> for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}
