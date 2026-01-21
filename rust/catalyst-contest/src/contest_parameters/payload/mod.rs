//! Contest Parameters payload type.

mod serde_election_public_key;

use std::ops::Deref;

use catalyst_voting::{crypto::group::GroupElement, vote_protocol::committee::ElectionPublicKey};
use chrono::{DateTime, Utc};

/// Content Parameters JSON payload type.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ContestParametersPayload {
    /// Contest start date
    pub(crate) start: DateTime<Utc>,
    /// Contest end date
    pub(crate) end: DateTime<Utc>,
    /// Contest choices
    pub(crate) choices: Choices,
    /// An election public key.
    #[serde(with = "serde_election_public_key")]
    pub(crate) election_public_key: ElectionPublicKey,
}

#[derive(Debug, Clone, Default)]
pub struct Choices(Vec<String>);

impl Deref for Choices {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ContestParametersPayload {
    fn default() -> Self {
        Self {
            start: DateTime::default(),
            end: DateTime::default(),
            choices: Choices::default(),
            election_public_key: GroupElement::zero().into(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Choices {
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
