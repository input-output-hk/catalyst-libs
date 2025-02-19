//! Catalyst Signed Document `section` field type defition.

use std::{fmt::Display, str::FromStr};

use coset::cbor::Value;
use serde::{Deserialize, Serialize};

/// 'section' field type defition, which is a JSON path string
#[derive(Clone, Debug, PartialEq)]
pub struct Section(jsonpath_rust::JsonPath<serde_json::Value>);

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Section {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Section {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let str = String::deserialize(deserializer)?;
        Ok(Self::from_str(&str).map_err(|e| serde::de::Error::custom(e))?)
    }
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            jsonpath_rust::JsonPath::<serde_json::Value>::from_str(&s)?,
        ))
    }
}

impl From<Section> for Value {
    fn from(value: Section) -> Self {
        Value::Text(value.to_string())
    }
}

impl TryFrom<&Value> for Section {
    type Error = anyhow::Error;

    fn try_from(val: &Value) -> anyhow::Result<Self> {
        let str = val
            .as_text()
            .ok_or(anyhow::anyhow!("Not a cbor string type"))?;
        Ok(Self::from_str(str)?)
    }
}
