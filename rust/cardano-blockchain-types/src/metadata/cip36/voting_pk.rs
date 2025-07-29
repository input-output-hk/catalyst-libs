//! Voting public key containing the public key and weight.

use ed25519_dalek::VerifyingKey;

/// Voting public key containing the public key and weight.
#[derive(Clone, Debug)]
pub struct VotingPubKey {
    /// Voting public key.
    voting_pk: Option<VerifyingKey>,
    /// Voting key associated weight.
    weight: u32,
}

impl VotingPubKey {
    /// Create a new voting public key.
    #[must_use]
    pub fn new(voting_pk: Option<VerifyingKey>, weight: u32) -> Self {
        Self { voting_pk, weight }
    }

    /// Get the voting public key.
    #[must_use]
    pub fn voting_pk(&self) -> Option<&VerifyingKey> {
        self.voting_pk.as_ref()
    }

    /// Get the voting key weight.
    #[must_use]
    pub fn weight(&self) -> u32 {
        self.weight
    }
}
