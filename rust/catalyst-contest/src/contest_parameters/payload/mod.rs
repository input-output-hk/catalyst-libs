//! Contest Parameters payload type.

use std::ops::Deref;

use chrono::{DateTime, Utc};

/// Content Parameters JSON payload type.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(crate) struct ContestParametersPayload {
    /// Contest start date
    pub(crate) start: DateTime<Utc>,
    /// Contest end date
    pub(crate) end: DateTime<Utc>,
    /// Contest choices
    pub(crate) choices: Choices,
}

#[derive(Debug, Clone, Default)]
pub struct Choices(Vec<String>);

impl Deref for Choices {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
