//! Catalyst Signed Document `section` field type definition.

use std::{fmt::Display, str::FromStr};

/// 'section' field type definition, which is a JSON path string
#[derive(Clone, Debug, PartialEq)]
pub struct Section(jsonpath_rust::JsonPath<serde_json::Value>);

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Section {
    type Err = jsonpath_rust::JsonPathParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        jsonpath_rust::JsonPath::<serde_json::Value>::from_str(s).map(Self)
    }
}

impl<C> minicbor::Encode<C> for Section {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(&self.0.to_string())?;
        Ok(())
    }
}

impl<'b, C> minicbor::Decode<'b, C> for Section {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let s = d.str()?;
        s.parse().map_err(minicbor::decode::Error::custom)
    }
}
