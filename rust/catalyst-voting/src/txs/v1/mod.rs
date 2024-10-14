//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

#![allow(unused_variables, dead_code)]

mod decoding;

use crate::{
    vote_protocol::voter::{proof::VoterProof, EncryptedVote},
    PublicKey,
};

/// A v1 (Jörmungandr) transaction struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    /// Vote plan id
    vote_plan_id: [u8; 32],
    /// Proposal index
    proposal_index: u8,
    /// Vote
    vote: Vote,
    /// Public key
    public_key: PublicKey,
}

/// Vote struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    /// Public voting choice
    Public(u8),
    /// Private (encrypted) voting choice
    Private(EncryptedVote, VoterProof),
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, any_with, Arbitrary, BoxedStrategy, Strategy};

    use super::*;
    use crate::SecretKey;

    impl Arbitrary for Tx {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<([u8; 32], u8, Vote, SecretKey)>()
                .prop_map(|(vote_plan_id, proposal_index, vote, s)| {
                    Tx {
                        vote_plan_id,
                        proposal_index,
                        vote,
                        public_key: s.public_key(),
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for Vote {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<bool>()
                .prop_flat_map(|b| {
                    if b {
                        any::<u8>().prop_map(Vote::Public).boxed()
                    } else {
                        any::<(u8, u8)>()
                            .prop_flat_map(|(s1, s2)| {
                                any_with::<(EncryptedVote, VoterProof)>((s1.into(), s2.into()))
                                    .prop_map(|(v, p)| Vote::Private(v, p))
                            })
                            .boxed()
                    }
                })
                .boxed()
        }
    }
}
