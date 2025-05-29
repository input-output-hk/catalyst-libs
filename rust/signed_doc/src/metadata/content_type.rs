//! Document Payload Content Type.

use serde::{Deserialize, Serialize};
use strum::{Display as EnumDisplay, EnumString, IntoStaticStr, VariantArray};

use super::utils::{transcode_ciborium_with, transcode_coset_with};

/// Payload Content Type.
// TODO: add custom parse error type when the [strum issue]([`issue`](https://github.com/Peternator7/strum/issues/430)) fix is merged.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    VariantArray,
    EnumString,
    EnumDisplay,
    IntoStaticStr,
    Serialize,
    Deserialize,
)]
#[serde(try_from = "&str", into = "&str")]
pub enum ContentType {
    /// 'application/cbor'
    #[strum(to_string = "application/cbor")]
    Cbor,
    /// 'application/json'
    #[strum(to_string = "application/json")]
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

    /// An error returned on [`minicbor::Decode::decode`] failure.
    fn decode_error(input: &str) -> minicbor::decode::Error {
        minicbor::decode::Error::message(format!(
            "Unsupported Content Type {input:?}, Supported only: {:?}",
            ContentType::VARIANTS
                .iter()
                .map(<&str>::from)
                .collect::<Vec<_>>()
        ))
    }
}

impl<C> minicbor::Encode<C> for ContentType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(<&str>::from(self))?.ok()
    }
}

impl<'b, C> minicbor::Decode<'b, C> for ContentType {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let s = d.str()?;
        s.parse().map_err(|_| Self::decode_error(s))
    }
}

impl TryFrom<&coset::cbor::Value> for ContentType {
    type Error = minicbor::decode::Error;

    fn try_from(val: &coset::cbor::Value) -> Result<Self, minicbor::decode::Error> {
        transcode_ciborium_with(val, &mut ())
    }
}

type CosetLabel = coset::RegisteredLabel<coset::iana::CoapContentFormat>;

impl TryFrom<&CosetLabel> for ContentType {
    type Error = minicbor::decode::Error;

    fn try_from(val: &CosetLabel) -> Result<Self, minicbor::decode::Error> {
        transcode_coset_with(val.clone(), &mut ())
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

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
