//! Document Payload Content Type.

use std::{str::FromStr, string::ToString};

use strum::VariantArray;

/// Payload Content Type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, VariantArray, strum_macros::Display)]
pub enum ContentType {
    /// `application/cbor`
    #[strum(to_string = "application/cbor")]
    Cbor,
    /// `application/cddl`
    #[strum(to_string = "application/cddl")]
    Cddl,
    /// `application/json`
    #[strum(to_string = "application/json")]
    Json,
    /// `application/json+schema`
    #[strum(to_string = "application/json+schema")]
    JsonSchema,
    /// `text/css; charset=utf-8`
    #[strum(to_string = "text/css; charset=utf-8")]
    Css,
    /// `text/css; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/css; charset=utf-8; template=handlebars")]
    CssTemplate,
    /// `text/html; charset=utf-8`
    #[strum(to_string = "text/html; charset=utf-8")]
    Html,
    /// `text/html; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/html; charset=utf-8; template=handlebars")]
    HtmlTemplate,
    /// `text/markdown; charset=utf-8`
    #[strum(to_string = "text/markdown; charset=utf-8")]
    Markdown,
    /// `text/markdown; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/markdown; charset=utf-8; template=handlebars")]
    MarkdownTemplate,
    /// `text/plain; charset=utf-8`
    #[strum(to_string = "text/plain; charset=utf-8")]
    Text,
    /// `text/plain; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/plain; charset=utf-8; template=handlebars")]
    TextTemplate,
}

impl FromStr for ContentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/cbor" => Ok(Self::Cbor),
            "application/cddl" => Ok(Self::Cddl),
            "application/json" => Ok(Self::Json),
            "application/json+schema" => Ok(Self::JsonSchema),
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

impl<'de> serde::Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for ContentType {
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

impl minicbor::Encode<()> for ContentType {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // encode as media types, not in CoAP Content-Formats
        e.str(self.to_string().as_str())?;
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for ContentType {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let p = d.position();
        match d.int() {
            // CoAP Content Format JSON
            Ok(val) if val == minicbor::data::Int::from(50_u8) => Ok(Self::Json),
            // CoAP Content Format CBOR
            Ok(val) if val == minicbor::data::Int::from(60_u8) => Ok(Self::Cbor),
            Ok(val) => {
                Err(minicbor::decode::Error::message(format!(
                    "unsupported CoAP Content Formats value: {val}"
                )))
            },
            Err(_) => {
                d.set_position(p);
                d.str()?.parse().map_err(minicbor::decode::Error::message)
            },
        }
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
            ContentType::from_str("application/cddl").unwrap(),
            ContentType::Cddl
        );
        assert_eq!(
            ContentType::from_str("application/json").unwrap(),
            ContentType::Json
        );
        assert_eq!(
            ContentType::from_str("application/json+schema").unwrap(),
            ContentType::JsonSchema
        );
        assert_eq!(
            "application/cbor".parse::<ContentType>().unwrap(),
            ContentType::Cbor
        );
        assert_eq!(
            "application/cddl".parse::<ContentType>().unwrap(),
            ContentType::Cddl
        );
        assert_eq!(
            "application/json".parse::<ContentType>().unwrap(),
            ContentType::Json
        );
        assert_eq!(
            "application/json+schema".parse::<ContentType>().unwrap(),
            ContentType::JsonSchema
        );
    }
}
