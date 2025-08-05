//! Catalyst Signed Document `section` field type definition.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// 'section' field type definition, which is a JSON path string
#[derive(Clone, Debug, PartialEq)]
pub struct Section(jsonpath_rust::JsonPath<serde_json::Value>);

impl Display for Section {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Section {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Section {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let str = String::deserialize(deserializer)?;
        Self::from_str(&str).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            jsonpath_rust::JsonPath::<serde_json::Value>::from_str(s)?,
        ))
    }
}

impl minicbor::Encode<()> for Section {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(self.to_string().as_str())?;
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Section {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        d.str()?.parse().map_err(minicbor::decode::Error::message)
    }
}
