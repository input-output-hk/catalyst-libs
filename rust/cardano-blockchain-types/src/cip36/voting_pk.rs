//! Voting public key containing the public key and weight.

use ed25519_dalek::VerifyingKey;

/// Voting public key containing the public key and weight.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VotingPubKey {
    /// Voting public key.
    pub voting_pk: VerifyingKey,
    /// Voting key associated weight.
    pub weight: u32,
}
