//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

mod decoding;

use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

use crate::{
    crypto::{
        ed25519::{sign, PrivateKey, PublicKey, Signature},
        hash::{digest::Digest, Blake2b512Hasher},
    },
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
#[must_use]
pub struct Tx {
    /// Vote plan id
    vote_plan_id: [u8; 32],
    /// Proposal index
    proposal_index: u8,
    /// Vote
    vote: VotePayload,
    /// Public key
    public_key: PublicKey,
    /// Transaction signature
    signature: Signature,
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
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_public(
        vote_plan_id: [u8; 32], proposal_index: u8, choice: u8, proposal_voting_options: u8,
        users_private_key: &PrivateKey,
    ) -> anyhow::Result<Self> {
        let vote = VotePayload::new_public(choice, proposal_voting_options)?;
        let signature = Self::sign(&vote_plan_id, proposal_index, &vote, users_private_key);
        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key: users_private_key.public_key(),
            signature,
        })
    }

    /// Generate a new `Tx` with public vote
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_private(
        vote_plan_id: [u8; 32], proposal_index: u8, choice: u8, proposal_voting_options: u8,
        election_public_key: &ElectionPublicKey, users_private_key: &PrivateKey,
    ) -> anyhow::Result<Self> {
        let vote = VotePayload::new_private(
            &vote_plan_id,
            choice,
            proposal_voting_options,
            election_public_key,
        )?;
        let signature = Self::sign(&vote_plan_id, proposal_index, &vote, users_private_key);

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key: users_private_key.public_key(),
            signature,
        })
    }

    /// Generate transaction signature
    fn sign(
        vote_plan_id: &[u8; 32], proposal_index: u8, vote: &VotePayload, private_key: &PrivateKey,
    ) -> Signature {
        let mut bytes = Vec::new();
        Self::bytes_to_sign(
            vote_plan_id,
            proposal_index,
            vote,
            &private_key.public_key(),
            &mut bytes,
        );
        let msg = Blake2b512Hasher::new()
            .chain_update(bytes.as_slice())
            .finalize();
        sign(private_key, msg.as_slice())
    }
}

#[allow(clippy::missing_docs_in_private_items)]
impl VotePayload {
    fn new_public(choice: u8, proposal_voting_options: u8) -> anyhow::Result<Self> {
        // Try to make a `Vote` just for applying underlying validation, which must be the same
        // even for public vote
        Vote::new(choice.into(), proposal_voting_options.into())?;
        Ok(Self::Public(choice))
    }

    fn new_private(
        vote_plan_id: &[u8; 32], choice: u8, proposal_voting_options: u8,
        election_public_key: &ElectionPublicKey,
    ) -> anyhow::Result<Self> {
        let vote = Vote::new(choice.into(), proposal_voting_options.into())?;

        let mut rng = ChaCha20Rng::from_entropy();
        let (encrypted_vote, randomness) = encrypt_vote(&vote, election_public_key, &mut rng);

        let vote_plan_id_hash = Blake2b512Hasher::new().chain_update(vote_plan_id);
        let commitment = VoterProofCommitment::from_hash(vote_plan_id_hash);

        let voter_proof = generate_voter_proof(
            &vote,
            encrypted_vote.clone(),
            randomness,
            election_public_key,
            &commitment,
            &mut rng,
        )?;

        Ok(Self::Private(encrypted_vote, voter_proof))
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;
    use crate::{crypto::ed25519::PrivateKey, vote_protocol::committee::ElectionSecretKey};

    #[proptest]
    fn tx_private_test(
        vote_plan_id: [u8; 32], proposal_index: u8, #[strategy(1u8..)] proposal_voting_options: u8,
        #[strategy(0..#proposal_voting_options)] choice: u8, users_private_key: PrivateKey,
        election_secret_key: ElectionSecretKey,
    ) {
        let election_public_key = election_secret_key.public_key();

        let _tx = Tx::new_private(
            vote_plan_id,
            proposal_index,
            choice,
            proposal_voting_options,
            &election_public_key,
            &users_private_key,
        )
        .unwrap();
    }
}
