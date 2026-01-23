//! Contest Parameters payload type.

mod serde_election_public_key;

use catalyst_voting::{crypto::group::GroupElement, vote_protocol::committee::ElectionPublicKey};
use chrono::{DateTime, Utc};

/// Content Parameters JSON payload type.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ContestParametersPayload {
    /// Contest start date
    pub(crate) start: DateTime<Utc>,
    /// Contest end date
    pub(crate) end: DateTime<Utc>,
    /// Contest snapshot taking date
    pub(crate) snapshot: DateTime<Utc>,
    /// Contest voting options
    pub(crate) options: VotingOptions,
    /// An election public key.
    #[serde(with = "serde_election_public_key")]
    pub(crate) election_public_key: ElectionPublicKey,
}

/// Contest Choices
#[derive(Debug, Clone, Default)]
pub struct VotingOptions(Vec<String>);

impl VotingOptions {
    /// Returns the number of voting options
    #[must_use]
    pub fn n_options(&self) -> usize {
        self.0.len()
    }
}

impl Default for ContestParametersPayload {
    fn default() -> Self {
        Self {
            start: DateTime::default(),
            end: DateTime::default(),
            snapshot: DateTime::default(),
            options: VotingOptions::default(),
            election_public_key: GroupElement::zero().into(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for VotingOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let res = Vec::<String>::deserialize(deserializer)?;

        if res.len() < 2 {
            return Err(serde::de::Error::custom(
                "It must be at least 2 choices, otherwise the contest does not make any sense.",
            ));
        }

        Ok(Self(res))
    }
}
