//! Document Payload Content Type.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use coset::iana::CoapContentFormat;
use serde::{de, Deserialize, Deserializer};
use strum::VariantArray;

/// Payload Content Type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, VariantArray)]
pub enum ContentType {
    /// 'application/cbor'
    Cbor,
    /// 'application/json'
    Json,
}

impl ContentType {
    /// Validates the provided `content` bytes to be a defined `ContentType`.
    pub(crate) fn validate(self, content: &[u8]) -> anyhow::Result<()> {
        match self {
            Self::Json => {
                if let Err(e) = serde_json::from_slice::<serde_json::Value>(content) {
                    anyhow::bail!("Invalid {self} content: {e}")
                }
            },
            Self::Cbor => {
                if let Err(e) = minicbor::decode::<minicbor::data::Token>(content) {
                    anyhow::bail!("Invalid {self} content: {e}")
                }
            },
        }
        Ok(())
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Cbor => write!(f, "application/cbor"),
            Self::Json => write!(f, "application/json"),
        }
    }
}

impl FromStr for ContentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/cbor" => Ok(Self::Cbor),
            "application/json" => Ok(Self::Json),
            _ => {
                anyhow::bail!(
                    "Unsupported Content Type: {s:?}, Supported only: {:?}",
                    ContentType::VARIANTS
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
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
            _ => {
                anyhow::bail!(
                    "Unsupported Content Type {value:?}, Supported only: {:?}",
                    ContentType::VARIANTS
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
        };
        Ok(content_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_validate_test() {
        let json_bytes = serde_json::to_vec(&serde_json::Value::Null).unwrap();
        assert!(ContentType::Json.validate(&json_bytes).is_ok());
        assert!(ContentType::Cbor.validate(&json_bytes).is_err());

        let cbor_bytes = minicbor::to_vec(minicbor::data::Token::Null).unwrap();
        assert!(ContentType::Json.validate(&cbor_bytes).is_err());
        assert!(ContentType::Cbor.validate(&cbor_bytes).is_ok());
    }

    #[test]
    fn content_type_string_test() {
        assert_eq!(
            ContentType::from_str("application/cbor").unwrap(),
            ContentType::Cbor
        );
        assert_eq!(
            ContentType::from_str("application/json").unwrap(),
            ContentType::Json
        );
        assert_eq!(
            "application/cbor".parse::<ContentType>().unwrap(),
            ContentType::Cbor
        );
        assert_eq!(
            "application/json".parse::<ContentType>().unwrap(),
            ContentType::Json
        );
    }
}
