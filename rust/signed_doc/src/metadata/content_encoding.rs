//! Document Payload Content Encoding.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{de, Deserialize, Deserializer};

/// Catalyst Signed Document Content Encoding Key.
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";

/// IANA `CoAP` Content Encoding.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContentEncoding {
    /// Brotli compression.format.
    Brotli,
}

impl Display for ContentEncoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Brotli => write!(f, "br"),
        }
    }
}

impl FromStr for ContentEncoding {
    type Err = anyhow::Error;

    fn from_str(encoding: &str) -> Result<Self, Self::Err> {
        match encoding {
            "br" => Ok(ContentEncoding::Brotli),
            _ => anyhow::bail!("Unsupported Content Encoding: {encoding:?}"),
        }
    }
}

impl<'de> Deserialize<'de> for ContentEncoding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl TryFrom<&coset::cbor::Value> for ContentEncoding {
    type Error = anyhow::Error;

    fn try_from(val: &coset::cbor::Value) -> anyhow::Result<ContentEncoding> {
        match val.as_text() {
            Some(encoding) => encoding.parse(),
            None => {
                anyhow::bail!("Expected Content Encoding to be a string");
            },
        }
    }
}
