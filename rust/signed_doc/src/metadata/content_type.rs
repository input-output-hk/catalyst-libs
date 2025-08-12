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
    CssHandlebars,
    /// `text/html; charset=utf-8`
    #[strum(to_string = "text/html; charset=utf-8")]
    Html,
    /// `text/html; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/html; charset=utf-8; template=handlebars")]
    HtmlHandlebars,
    /// `text/markdown; charset=utf-8`
    #[strum(to_string = "text/markdown; charset=utf-8")]
    Markdown,
    /// `text/markdown; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/markdown; charset=utf-8; template=handlebars")]
    MarkdownHandlebars,
    /// `text/plain; charset=utf-8`
    #[strum(to_string = "text/plain; charset=utf-8")]
    Plain,
    /// `text/plain; charset=utf-8; template=handlebars`
    #[strum(to_string = "text/plain; charset=utf-8; template=handlebars")]
    PlainHandlebars,
}

impl FromStr for ContentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/cbor" => Ok(Self::Cbor),
            "application/cddl" => Ok(Self::Cddl),
            "application/json" => Ok(Self::Json),
            "application/json+schema" => Ok(Self::JsonSchema),
            "text/css; charset=utf-8" => Ok(Self::Css),
            "text/css; charset=utf-8; template=handlebars" => Ok(Self::CssHandlebars),
            "text/html; charset=utf-8" => Ok(Self::Html),
            "text/html; charset=utf-8; template=handlebars" => Ok(Self::HtmlHandlebars),
            "text/markdown; charset=utf-8" => Ok(Self::Markdown),
            "text/markdown; charset=utf-8; template=handlebars" => Ok(Self::MarkdownHandlebars),
            "text/plain; charset=utf-8" => Ok(Self::Plain),
            "text/plain; charset=utf-8; template=handlebars" => Ok(Self::PlainHandlebars),
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

impl TryFrom<u64> for ContentType {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        // https://www.iana.org/assignments/core-parameters/core-parameters.xhtml#content-formats
        match value {
            0 => Ok(Self::Plain),
            50 => Ok(Self::Json),
            60 => Ok(Self::Cbor),
            20000 => Ok(Self::Css),
            _ => {
                anyhow::bail!(
                    "Unsupported CoAP Content-Format: {value}, Supported only: 0, 50, 60, 20000",
                )
            },
        }
    }
}

impl<'de> serde::Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
            Ok(val) => {
                let val: u64 = val.try_into().map_err(minicbor::decode::Error::custom)?;
                Self::try_from(val).map_err(minicbor::decode::Error::message)
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
        let entries = [
            ("application/cbor", ContentType::Cbor),
            ("application/cddl", ContentType::Cddl),
            ("application/json", ContentType::Json),
            ("application/json+schema", ContentType::JsonSchema),
            ("text/css; charset=utf-8", ContentType::Css),
            (
                "text/css; charset=utf-8; template=handlebars",
                ContentType::CssHandlebars,
            ),
            ("text/html; charset=utf-8", ContentType::Html),
            (
                "text/html; charset=utf-8; template=handlebars",
                ContentType::HtmlHandlebars,
            ),
            ("text/markdown; charset=utf-8", ContentType::Markdown),
            (
                "text/markdown; charset=utf-8; template=handlebars",
                ContentType::MarkdownHandlebars,
            ),
            ("text/plain; charset=utf-8", ContentType::Plain),
            (
                "text/plain; charset=utf-8; template=handlebars",
                ContentType::PlainHandlebars,
            ),
        ];

        for (raw_str, variant) in entries {
            assert_eq!(ContentType::from_str(raw_str).unwrap(), variant);
        }
        for (raw_str, variant) in entries {
            assert_eq!(raw_str.parse::<ContentType>().unwrap(), variant);
        }
    }
}
