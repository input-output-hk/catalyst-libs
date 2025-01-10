//! Document Payload Content Type.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use coset::iana::CoapContentFormat;
use serde::{de, Deserialize, Deserializer};

/// Payload Content Type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// 'application/cbor'
    Cbor,
    /// 'application/json'
    Json,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Json
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Cbor => write!(f, "cbor"),
            Self::Json => write!(f, "json"),
        }
    }
}

impl FromStr for ContentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cbor" => Ok(Self::Cbor),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("Unsupported Content Type: {s:?}"),
        }
    }
}

impl<'de> Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl From<ContentType> for CoapContentFormat {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::Cbor => Self::Cbor,
            ContentType::Json => Self::Json,
        }
    }
}

impl TryFrom<&coset::ContentType> for ContentType {
    type Error = anyhow::Error;

    fn try_from(value: &coset::ContentType) -> Result<Self, Self::Error> {
        let content_type = match value {
            coset::ContentType::Assigned(CoapContentFormat::Json) => ContentType::Json,
            coset::ContentType::Assigned(CoapContentFormat::Cbor) => ContentType::Cbor,
            _ => anyhow::bail!("Unsupported Content Type {value:?}"),
        };
        Ok(content_type)
    }
}
