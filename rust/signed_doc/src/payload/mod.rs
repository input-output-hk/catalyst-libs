//! Catalyst Signed Document JSON Payload

use std::fmt::{Display, Formatter};

/// JSON Content
#[derive(Debug, Default)]
pub struct Content(serde_json::Value);

impl From<serde_json::Value> for Content {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
