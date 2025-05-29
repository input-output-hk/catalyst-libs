//! Catalyst Signed Document `section` field type definition.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use super::utils::transcode_ciborium_with;

/// 'section' field type definition, which is a JSON path string
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "&str")]
pub struct Section(jsonpath_rust::JsonPath<serde_json::Value>);

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Section> for String {
    fn from(value: Section) -> Self {
        value.to_string()
    }
}

impl FromStr for Section {
    type Err = jsonpath_rust::JsonPathParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl TryFrom<&str> for Section {
    type Error = jsonpath_rust::JsonPathParserError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<C> minicbor::Encode<C> for Section {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(&self.0.to_string())?.ok()
    }
}

impl<'b, C> minicbor::Decode<'b, C> for Section {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        d.str()?.parse().map_err(minicbor::decode::Error::custom)
    }
}

impl TryFrom<&coset::cbor::Value> for Section {
    type Error = minicbor::decode::Error;

    fn try_from(val: &coset::cbor::Value) -> Result<Self, minicbor::decode::Error> {
        transcode_ciborium_with(val, &mut ())
    }
}
