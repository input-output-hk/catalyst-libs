//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)
//!
//! ```rust
//! use catalyst_voting::{
//!     crypto::{default_rng, ed25519::PrivateKey},
//!     txs::v1::Tx,
//!     vote_protocol::committee::ElectionSecretKey,
//! };
//!
//! let vote_plan_id = [0u8; 32];
//! let proposal_index = 0u8;
//!
//! let voting_options = 3;
//! let choice = 1;
//!
//! let users_private_key = PrivateKey::random(&mut default_rng());
//! let election_public_key = ElectionSecretKey::random_with_default_rng().public_key();
//!
//! let public_tx = Tx::new_public(
//!     vote_plan_id,
//!     proposal_index,
//!     voting_options,
//!     choice,
//!     &users_private_key,
//! )
//! .unwrap();
//!
//! let private_tx = Tx::new_private_with_default_rng(
//!     vote_plan_id,
//!     proposal_index,
//!     voting_options,
//!     choice,
//!     &election_public_key,
//!     &users_private_key,
//! )
//! .unwrap();
//! ```

mod decoding;

use anyhow::ensure;
use rand_core::CryptoRngCore;

use crate::{
    crypto::{
        default_rng,
        ed25519::{sign, verify_signature, PrivateKey, PublicKey, Signature},
        hash::{digest::Digest, Blake2b256Hasher, Blake2b512Hasher},
    },
    vote_protocol::{
        committee::ElectionPublicKey,
        voter::{
            encrypt_vote_with_default_rng,
            proof::{generate_voter_proof, verify_voter_proof, VoterProof, VoterProofCommitment},
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
    /// Generate a new `Tx` with public vote.
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_public(
        vote_plan_id: [u8; 32], proposal_index: u8, voting_options: u8, choice: u8,
        users_private_key: &PrivateKey,
    ) -> anyhow::Result<Self> {
        let vote = VotePayload::new_public(choice, voting_options)?;
        let signature = Self::sign(&vote_plan_id, proposal_index, &vote, users_private_key);
        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key: users_private_key.public_key(),
            signature,
        })
    }

    /// Generate a new `Tx` with public vote.
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_private<R: CryptoRngCore>(
        vote_plan_id: [u8; 32], proposal_index: u8, voting_options: u8, choice: u8,
        election_public_key: &ElectionPublicKey, users_private_key: &PrivateKey, rng: &mut R,
    ) -> anyhow::Result<Self> {
        let vote = VotePayload::new_private(
            &vote_plan_id,
            choice,
            voting_options,
            election_public_key,
            rng,
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

    /// Generate a new `Tx` with public vote with `crypto::default_rng`.
    ///
    /// # Errors
    ///   - Invalid voting choice
    pub fn new_private_with_default_rng(
        vote_plan_id: [u8; 32], proposal_index: u8, voting_options: u8, choice: u8,
        election_public_key: &ElectionPublicKey, users_private_key: &PrivateKey,
    ) -> anyhow::Result<Self> {
        Self::new_private(
            vote_plan_id,
            proposal_index,
            voting_options,
            choice,
            election_public_key,
            users_private_key,
            &mut default_rng(),
        )
    }

    /// Returns `true` if the vote is public
    #[must_use]
    pub fn is_public(&self) -> bool {
        matches!(self.vote, VotePayload::Public(_))
    }

    /// Returns `true` if the vote is private
    #[must_use]
    pub fn is_private(&self) -> bool {
        matches!(self.vote, VotePayload::Private(_, _))
    }

    /// Verify transaction signature
    ///
    /// # Errors
    ///   - Invalid signature
    pub fn verify_signature(&self) -> anyhow::Result<()> {
        let bytes = Self::bytes_to_sign(
            &self.vote_plan_id,
            self.proposal_index,
            &self.vote,
            &self.public_key,
        );
        ensure!(
            verify_signature(&self.public_key, &bytes, &self.signature),
            "Invalid signature."
        );
        Ok(())
    }

    /// Verify transaction proof of the private vote.
    /// If vote is public it returns `Ok(())`
    ///
    /// # Errors
    ///   - Invalid proof
    pub fn verify_proof(&self, election_public_key: &ElectionPublicKey) -> anyhow::Result<()> {
        if let VotePayload::Private(encrypted_vote, proof) = &self.vote {
            let vote_plan_id_hash = Blake2b512Hasher::new().chain_update(self.vote_plan_id);
            let commitment = VoterProofCommitment::from_hash(vote_plan_id_hash);
            ensure!(
                verify_voter_proof(
                    encrypted_vote.clone(),
                    election_public_key,
                    &commitment,
                    proof,
                ),
                "Invalid proof."
            );
        }
        Ok(())
    }

    /// Generate transaction signature
    fn sign(
        vote_plan_id: &[u8; 32], proposal_index: u8, vote: &VotePayload, private_key: &PrivateKey,
    ) -> Signature {
        let bytes = Self::bytes_to_sign(
            vote_plan_id,
            proposal_index,
            vote,
            &private_key.public_key(),
        );
        sign(private_key, &bytes)
    }

    /// Generate bytes to be signed.
    /// A Blake2b256 hash of the transaction body
    fn bytes_to_sign(
        vote_plan_id: &[u8; 32], proposal_index: u8, vote: &VotePayload, public_key: &PublicKey,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        Self::tx_body_decode(vote_plan_id, proposal_index, vote, public_key, &mut bytes);
        Blake2b256Hasher::new()
            .chain_update(bytes.as_slice())
            .finalize()
            .to_vec()
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

    fn new_private<R: CryptoRngCore>(
        vote_plan_id: &[u8; 32], choice: u8, proposal_voting_options: u8,
        election_public_key: &ElectionPublicKey, rng: &mut R,
    ) -> anyhow::Result<Self> {
        let vote = Vote::new(choice.into(), proposal_voting_options.into())?;

        let (encrypted_vote, randomness) =
            encrypt_vote_with_default_rng(&vote, election_public_key);

        let vote_plan_id_hash = Blake2b512Hasher::new().chain_update(vote_plan_id);
        let commitment = VoterProofCommitment::from_hash(vote_plan_id_hash);

        let voter_proof = generate_voter_proof(
            &vote,
            encrypted_vote.clone(),
            randomness,
            election_public_key,
            &commitment,
            rng,
        )?;

        Ok(Self::Private(encrypted_vote, voter_proof))
    }

    // #[allow(clippy::cast_possible_truncation, dead_code)]
    // fn choice(&self, secret_key: &ElectionSecretKey) -> anyhow::Result<u8> {
    //     match self {
    //         Self::Public(choice) => Ok(*choice),
    //         Self::Private(vote, _) => {
    //             // Making a tally and decryption tally procedure on one vote to retrieve
    // the             // original voting choice.
    //             // Assuming that the voting power argument must be equals to 1.
    //             let setup = DecryptionTallySetup::new(1)?;
    //             for voting_option in 0..vote.voting_options() {
    //                 let tally = tally(voting_option, &[vote.clone()], &[1])?;
    //                 let choice_for_voting_option = decrypt_tally(&tally, secret_key,
    // &setup)?;                 if choice_for_voting_option == 1 {
    //                     return Ok(voting_option as u8);
    //                 }
    //             }
    //             bail!("Invalid encrypted vote, not a unit vector");
    //         },
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;
    use crate::{crypto::ed25519::PrivateKey, vote_protocol::committee::ElectionSecretKey};

    #[proptest]
    fn tx_test(
        vote_plan_id: [u8; 32], proposal_index: u8, #[strategy(1u8..5)] voting_options: u8,
        #[strategy(0..#voting_options)] choice: u8, users_private_key: PrivateKey,
        election_secret_key: ElectionSecretKey,
    ) {
        let election_public_key = election_secret_key.public_key();

        let tx = Tx::new_public(
            vote_plan_id,
            proposal_index,
            voting_options,
            choice,
            &users_private_key,
        )
        .unwrap();
        assert!(tx.is_public());
        assert!(!tx.is_private());
        tx.verify_signature().unwrap();
        tx.verify_proof(&election_public_key).unwrap();

        let tx = Tx::new_private_with_default_rng(
            vote_plan_id,
            proposal_index,
            voting_options,
            choice,
            &election_public_key,
            &users_private_key,
        )
        .unwrap();
        assert!(!tx.is_public());
        assert!(tx.is_private());
        tx.verify_signature().unwrap();
        tx.verify_proof(&election_public_key).unwrap();
    }
}
