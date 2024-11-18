//! A Catalyst public vote transaction v2 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/#public-vote)

use std::ops::{Deref, DerefMut};

use crate::gen_tx::{GeneralizedTx, Uuid};

mod decoding;

/// A public vote tx struct.
pub struct PublicTx(GeneralizedTx<Choice, Proof, PropId>);

/// A public voting choice struct.
pub struct Choice(pub u64);

/// A public voting proof struct, CBOR `undefined`.
pub struct Proof;

/// A public voting proposal id struct.
pub struct PropId(pub Uuid);

impl Deref for PublicTx {
    type Target = GeneralizedTx<Choice, Proof, PropId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PublicTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}