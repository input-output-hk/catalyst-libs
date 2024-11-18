//! A Catalyst public vote transaction v2 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/#public-vote)

use std::ops::{Deref, DerefMut};

use crate::{
    gen_tx::{GeneralizedTx, Uuid},
    Cbor,
};

mod decoding;

/// A public vote tx struct.
pub struct PublicTx<VoteDataT>(GeneralizedTx<Choice, Proof, PropId, VoteDataT>)
where VoteDataT: for<'a> Cbor<'a>;

/// A public voting choice struct.
pub struct Choice(pub u64);

/// A public voting proof struct, CBOR `undefined`.
pub struct Proof;

/// A public voting proposal id struct.
pub struct PropId(pub Uuid);

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
