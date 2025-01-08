//! Catalyst Signed Document JSON Payload

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer};

mod json;

pub use json::Json as JsonContent;

/// JSON Content
#[derive(Debug)]
pub struct Content<T>(T);

impl<T> From<T> for Content<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display> Display for Content<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<'de, T: serde::Deserialize<'de>> Deserialize<'de> for Content<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        T::deserialize(deserializer).map(std::convert::Into::into)
    }
}
