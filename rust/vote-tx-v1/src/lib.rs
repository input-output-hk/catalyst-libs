//! A Catalyst v1 (Jörmungandr) vote transaction object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v1/)
//!
//! ```rust
//! use catalyst_voting::{
//!     crypto::{ed25519::PrivateKey, rng::default_rng},
//!     vote_protocol::committee::ElectionSecretKey,
//! };
//! use vote_tx_v1::Tx;
//!
//! let vote_plan_id = [0u8; 32];
//! let proposal_index = 0u8;
//!
//! let voting_options = 3;
//! let choice = 1;
//!
//! let users_private_key = PrivateKey::random(&mut default_rng());
//! let election_secret_key = ElectionSecretKey::random_with_default_rng();
//! let election_public_key = election_secret_key.public_key();
//!
//! let public_tx = Tx::new_public(
//!     vote_plan_id,
//!     proposal_index,
//!     voting_options,
//!     choice,
//!     &users_private_key,
//! )
//! .unwrap();
//! public_tx.verify_signature().unwrap();
//! let tx_choice = public_tx.public_choice().unwrap();
//! assert_eq!(tx_choice, choice);
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
//! private_tx.verify_signature().unwrap();
//! private_tx.verify_proof(&election_public_key).unwrap();
//! let tx_choice = private_tx.private_choice(&election_secret_key).unwrap();
//! assert_eq!(tx_choice, choice);
//! ```

mod decoding;
mod utils;

use anyhow::ensure;
use catalyst_voting::{
    crypto::{
        ed25519::{sign, verify_signature, PrivateKey, PublicKey, Signature},
        hash::{digest::Digest, Blake2b256Hasher, Blake2b512Hasher},
        rng::{default_rng, rand_core::CryptoRngCore},
    },
    vote_protocol::{
        committee::{ElectionPublicKey, ElectionSecretKey},
        voter::{
            decrypt_vote, encrypt_vote,
            proof::{generate_voter_proof, verify_voter_proof, VoterProof, VoterProofCommitment},
            EncryptedVote, Vote,
        },
    },
};

/// A v1 (Jörmungandr) vote transaction struct
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
/// Contains all necessary information for the valid vote.
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
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        voting_options: u8,
        choice: u8,
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
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        voting_options: u8,
        choice: u8,
        election_public_key: &ElectionPublicKey,
        users_private_key: &PrivateKey,
        rng: &mut R,
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
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        voting_options: u8,
        choice: u8,
        election_public_key: &ElectionPublicKey,
        users_private_key: &PrivateKey,
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

    /// Returns public voting choice.
    ///
    /// # Errors
    ///   - Not a public vote
    pub fn public_choice(&self) -> anyhow::Result<u8> {
        if let VotePayload::Public(choice) = &self.vote {
            Ok(*choice)
        } else {
            Err(anyhow::anyhow!("Not a public vote"))
        }
    }

    /// Returns private voting choice.
    ///
    /// # Errors
    ///   - Not a private vote
    #[allow(clippy::cast_possible_truncation)]
    pub fn private_choice(
        &self,
        secret_key: &ElectionSecretKey,
    ) -> anyhow::Result<u8> {
        if let VotePayload::Private(vote, _) = &self.vote {
            let vote = decrypt_vote(vote, secret_key)?;
            let choice = vote.choice() as u8;
            Ok(choice)
        } else {
            Err(anyhow::anyhow!("Not a private vote"))
        }
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
    pub fn verify_proof(
        &self,
        election_public_key: &ElectionPublicKey,
    ) -> anyhow::Result<()> {
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
        vote_plan_id: &[u8; 32],
        proposal_index: u8,
        vote: &VotePayload,
        private_key: &PrivateKey,
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
        vote_plan_id: &[u8; 32],
        proposal_index: u8,
        vote: &VotePayload,
        public_key: &PublicKey,
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
    fn new_public(
        choice: u8,
        proposal_voting_options: u8,
    ) -> anyhow::Result<Self> {
        // Try to make a `Vote` just for applying underlying validation, which must be the same
        // even for public vote
        Vote::new(choice.into(), proposal_voting_options.into())?;
        Ok(Self::Public(choice))
    }

    fn new_private<R: CryptoRngCore>(
        vote_plan_id: &[u8; 32],
        choice: u8,
        proposal_voting_options: u8,
        election_public_key: &ElectionPublicKey,
        rng: &mut R,
    ) -> anyhow::Result<Self> {
        let vote = Vote::new(choice.into(), proposal_voting_options.into())?;

        let (encrypted_vote, randomness) = encrypt_vote(&vote, election_public_key, rng);

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
}

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use catalyst_voting::{
        crypto::ed25519::PrivateKey, vote_protocol::committee::ElectionSecretKey,
    };
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn tx_test(
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        #[strategy(1u8..5)] voting_options: u8,
        #[strategy(0..#voting_options)] choice: u8,
    ) {
        let users_private_key = PrivateKey::random_with_default_rng();
        let election_secret_key = ElectionSecretKey::random_with_default_rng();
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
        assert_eq!(tx.public_choice().unwrap(), choice);
        assert!(tx.private_choice(&election_secret_key).is_err());

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
        assert_eq!(tx.private_choice(&election_secret_key).unwrap(), choice);
        assert!(tx.public_choice().is_err());
    }
}
