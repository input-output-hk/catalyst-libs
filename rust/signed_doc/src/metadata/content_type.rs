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

impl TryFrom<&coset::ContentType> for ContentType {
    type Error = anyhow::Error;

    fn try_from(value: &coset::ContentType) -> Result<Self, Self::Error> {
        match value {
            coset::ContentType::Assigned(CoapContentFormat::Json) => Ok(ContentType::Json),
            coset::ContentType::Assigned(CoapContentFormat::Cbor) => Ok(ContentType::Cbor),
            coset::ContentType::Text(str) => str.parse(),
            coset::RegisteredLabel::Assigned(_) => {
                anyhow::bail!(
                    "Unsupported Content Type: {value:?}, Supported only: {:?}",
                    ContentType::VARIANTS
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
        }
    }
}

impl minicbor::Encode<()> for ContentType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // encode as media types, not in CoAP Content-Formats
        e.str(self.to_string().as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
