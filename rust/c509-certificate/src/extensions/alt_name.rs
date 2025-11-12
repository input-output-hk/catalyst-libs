//! C509 Alternative Name uses for Subject Alternative Name extension and
//! Issuer Alternative Name extension.

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};
use serde::{Deserialize, Serialize};

use crate::{
    general_names::{
        GeneralNames,
        general_name::{GeneralName, GeneralNameTypeRegistry, GeneralNameValue},
    },
    helper::{
        decode::{decode_datatype, decode_helper},
        encode::encode_helper,
    },
};

/// Alternative Name extension.
/// Can be interpreted as a `GeneralNames / text`
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AlternativeName(GeneralNamesOrText);

impl AlternativeName {
    /// Create a new instance of `AlternativeName` given value.
    #[must_use]
    pub fn new(value: GeneralNamesOrText) -> Self {
        Self(value)
    }

    /// Get the general name which can be general names or text.
    #[must_use]
    pub fn general_name(&self) -> &GeneralNamesOrText {
        &self.0
    }
}

impl Encode<()> for AlternativeName {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for AlternativeName {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        GeneralNamesOrText::decode(d, ctx).map(AlternativeName::new)
    }
}

// ------------------GeneralNamesOrText--------------------

/// Enum for type that can be a `GeneralNames` or a text use in `AlternativeName`.
/// Type `Text` is also considered as a `GeneralNames` with only 1 `DNSName` as
/// a special case.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneralNamesOrText {
    /// A value of `GeneralNames`.
    GeneralNames(GeneralNames),
    /// A text string.
    Text(String),
}

impl Encode<()> for GeneralNamesOrText {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            GeneralNamesOrText::GeneralNames(gns) => {
                let gn = gns
                    .general_names()
                    .first()
                    .ok_or(minicbor::encode::Error::message("General Names is empty"))?;
                // Check whether there is only 1 item in the array which is a DNSName
                if gns.general_names().len() == 1 && gn.gn_type().is_dns_name() {
                    gn.gn_value().encode(e, ctx)?;
                } else {
                    gns.encode(e, ctx)?;
                }
            },
            GeneralNamesOrText::Text(text) => {
                encode_helper(e, "Alternative Name - General Name Text", ctx, text)?;
            },
        }
        Ok(())
    }
}

impl Decode<'_, ()> for GeneralNamesOrText {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        match decode_datatype(d, "Alternative Name - General Names")? {
            // If it is a string it is a GeneralNames with only 1 DNSName
            minicbor::data::Type::String => {
                let gn_dns = GeneralName::new(
                    GeneralNameTypeRegistry::DNSName,
                    GeneralNameValue::Text(decode_helper(
                        d,
                        "Alternative Name - General Name Text",
                        ctx,
                    )?),
                );
                let mut gns = GeneralNames::new();
                gns.add_general_name(gn_dns);
                Ok(GeneralNamesOrText::GeneralNames(gns))
            },
            minicbor::data::Type::Array => {
                Ok(GeneralNamesOrText::GeneralNames(GeneralNames::decode(
                    d, ctx,
                )?))
            },
            _ => {
                Err(minicbor::decode::Error::message(
                    "Invalid type for AlternativeName",
                ))
            },
        }
    }
}

// ------------------Test----------------------

#[cfg(test)]
mod test_alt_name {
    use super::*;
    use crate::general_names::general_name::{
        GeneralName, GeneralNameTypeRegistry, GeneralNameValue,
    };

    #[test]
    fn encode_only_dns() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        let mut gns = GeneralNames::new();
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));
        let alt_name = AlternativeName::new(GeneralNamesOrText::GeneralNames(gns));
        alt_name
            .encode(&mut encoder, &mut ())
            .expect("Failed to encode AlternativeName");
        // "example.com": 0x6b6578616d706c652e636f6d
        assert_eq!(hex::encode(buffer.clone()), "6b6578616d706c652e636f6d");

        let mut decoder = Decoder::new(&buffer);
        let decoded_alt_name = AlternativeName::decode(&mut decoder, &mut ())
            .expect("Failed to decode Alternative Name");
        assert_eq!(decoded_alt_name, alt_name);
    }

    #[test]
    fn encode_decode_text() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let alt_name = AlternativeName::new(GeneralNamesOrText::Text("example.com".to_string()));
        alt_name
            .encode(&mut encoder, &mut ())
            .expect("Failed to encode AlternativeName");
        // "example.com": 0x6b6578616d706c652e636f6d
        assert_eq!(hex::encode(buffer.clone()), "6b6578616d706c652e636f6d");

        // If only text, it should be GeneralNames with only 1 DNSName
        let mut gns = GeneralNames::new();
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));

        let mut decoder = Decoder::new(&buffer);
        let decoded_alt_name = AlternativeName::decode(&mut decoder, &mut ())
            .expect("Failed to decode Alternative Name");
        assert_eq!(
            decoded_alt_name,
            AlternativeName::new(GeneralNamesOrText::GeneralNames(gns))
        );
    }
}
