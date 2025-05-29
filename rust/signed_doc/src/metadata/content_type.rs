//! Document Payload Content Type.

use strum::{AsRefStr, Display as EnumDisplay, EnumString, VariantArray};

/// Payload Content Type.
// TODO: add custom parse error type when the [strum issue]([`issue`](https://github.com/Peternator7/strum/issues/430)) fix is merged.
#[derive(Debug, Copy, Clone, PartialEq, Eq, VariantArray, EnumString, EnumDisplay, AsRefStr)]
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
                if let Err(e) = serde_json::from_slice::<&serde_json::value::RawValue>(content) {
                    anyhow::bail!("Invalid {self} content: {e}")
                }
            },
            Self::Cbor => {
                if let Err(e) =
                    decode_any_to_end(&mut minicbor::Decoder::new(content), "signed doc content")
                {
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
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
        ))
    }
}

impl<C> minicbor::Encode<C> for ContentType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(self.as_ref())?.ok()
    }
}

impl<'b, C> minicbor::Decode<'b, C> for ContentType {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let s = d.str()?;
        s.parse().map_err(|_| Self::decode_error(s))
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
