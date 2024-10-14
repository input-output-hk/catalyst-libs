//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

mod decoding;

use curve25519_dalek::digest::Update;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

use crate::{
    crypto::hash::Blake2b512Hasher,
    vote_protocol::{
        committee::ElectionPublicKey,
        voter::{
            encrypt_vote,
            proof::{generate_voter_proof, VoterProof, VoterProofCommitment},
            EncryptedVote, Vote,
        },
    },
};

/// A v1 (Jörmungandr) transaction struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    /// Vote plan id
    vote_plan_id: [u8; 32],
    /// Proposal index
    proposal_index: u8,
    /// Vote
    vote: VotePayload,
    // /// Public key
    // public_key: PublicKey,
}

/// Vote payload struct.
/// Contains all necesarry information for the valid vote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VotePayload {
    /// Public voting choice
    Public(u8),
    /// Private (encrypted) voting choice
    Private(EncryptedVote, VoterProof),
}

impl Tx {
    /// Generate a new `Tx` with public vote
    #[must_use]
    pub fn new_public(vote_plan_id: [u8; 32], proposal_index: u8, choice: u8) -> Self {
        Self {
            vote_plan_id,
            proposal_index,
            vote: VotePayload::Public(choice),
        }
    }

    /// Generate a new `Tx` with public vote
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_private(
        vote_plan_id: [u8; 32], proposal_index: u8, proposal_voting_options: u8, choice: u8,
        election_public_key: &ElectionPublicKey,
    ) -> anyhow::Result<Self> {
        let vote = Vote::new(choice.into(), proposal_voting_options.into())?;

        let mut rng = ChaCha20Rng::from_entropy();
        let (encrypted_vote, randomness) = encrypt_vote(&vote, election_public_key, &mut rng);

        let vote_plan_id_hash = Blake2b512Hasher::new().chain(vote_plan_id);
        let commitment = VoterProofCommitment::from_hash(vote_plan_id_hash);

        let voter_proof = generate_voter_proof(
            &vote,
            encrypted_vote.clone(),
            randomness,
            election_public_key,
            &commitment,
            &mut rng,
        )?;

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote: VotePayload::Private(encrypted_vote, voter_proof),
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, any_with, Arbitrary, BoxedStrategy, Strategy};
    use test_strategy::proptest;

    use super::*;
    use crate::vote_protocol::committee::ElectionSecretKey;

    impl Arbitrary for Tx {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<([u8; 32], u8, VotePayload)>()
                .prop_map(|(vote_plan_id, proposal_index, vote)| {
                    Tx {
                        vote_plan_id,
                        proposal_index,
                        vote,
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for VotePayload {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<bool>()
                .prop_flat_map(|b| {
                    if b {
                        any::<u8>().prop_map(VotePayload::Public).boxed()
                    } else {
                        any::<(u8, u8)>()
                            .prop_flat_map(|(s1, s2)| {
                                any_with::<(EncryptedVote, VoterProof)>((s1.into(), s2.into()))
                                    .prop_map(|(v, p)| VotePayload::Private(v, p))
                            })
                            .boxed()
                    }
                })
                .boxed()
        }
    }

    #[proptest]
    fn tx_private_test(
        vote_plan_id: [u8; 32], proposal_index: u8, #[strategy(1u8..)] proposal_voting_options: u8,
        #[strategy(0..#proposal_voting_options)] choice: u8,
        election_secret_key: ElectionSecretKey,
    ) {
        let election_public_key = election_secret_key.public_key();

        Tx::new_private(
            vote_plan_id,
            proposal_index,
            proposal_voting_options,
            choice,
            &election_public_key,
        )
        .unwrap();
    }
}
