//! C509 Attribute
//!
//! ```cddl
//! Attribute = ( attributeType: int, attributeValue: text ) //
//!             ( attributeType: ~oid, attributeValue: bytes ) //
//! ```
//!
//! In some case attributeValue can have multiple values.
//!
//! ```cddl
//! Attributes = ( attributeType: int, attributeValue: [+text] ) //
//!              ( attributeType: ~oid, attributeValue: [+bytes] )
//! ```
//!
//! For more information about Attribute,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)

use std::str::FromStr;

use asn1_rs::Oid;
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Deserializer, Serialize};

use super::data::{get_oid_from_int, ATTRIBUTES_LOOKUP};
use crate::{
    helper::{
        decode::{decode_array_len, decode_datatype, decode_helper},
        encode::{encode_array_len, encode_helper},
    },
    oid::{C509oid, C509oidRegistered},
};
/// A struct of C509 `Attribute`
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// A registered OID of C509 `Attribute`.
    registered_oid: C509oidRegistered,
    /// A flag to indicate whether the value can have multiple value.
    multi_value: bool,
    /// A value of C509 `Attribute` can be a vector of text or bytes.
    value: Vec<AttributeValue>,
}

impl Attribute {
    /// Create a new instance of `Attribute`.
    #[must_use]
    pub fn new(oid: Oid<'static>) -> Self {
        Self {
            registered_oid: C509oidRegistered::new(oid, ATTRIBUTES_LOOKUP.get_int_to_oid_table()),
            multi_value: false,
            value: Vec::new(),
        }
    }

    /// Get the value of `Attribute`.
    #[must_use]
    pub fn value(&self) -> &[AttributeValue] {
        &self.value
    }

    /// Get the registered OID of `Attribute`.
    pub(crate) fn registered_oid(&self) -> &C509oidRegistered {
        &self.registered_oid
    }

    /// Add a value to `Attribute`.
    pub fn add_value(&mut self, value: AttributeValue) {
        self.value.push(value);
    }

    /// Set whether `Attribute` can have multiple value.
    pub(crate) fn set_multi_value(mut self) -> Self {
        self.multi_value = true;
        self
    }
}

/// A helper struct for deserialize and serialize `Attribute`.
#[derive(Debug, Deserialize, Serialize)]
struct Helper {
    /// An OID value in string.
    oid: String,
    /// A value of C509 `Attribute` can be a vector of text or bytes.
    value: Vec<AttributeValue>,
}

impl<'de> Deserialize<'de> for Attribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let helper = Helper::deserialize(deserializer)?;
        let oid =
            Oid::from_str(&helper.oid).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        let mut attr = Attribute::new(oid);
        for value in helper.value {
            attr.add_value(value);
        }
        Ok(attr)
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let helper = Helper {
            oid: self.registered_oid().c509_oid().oid().to_string(),
            value: self.value.clone(),
        };
        helper.serialize(serializer)
    }
}

impl Encode<()> for Attribute {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // Encode CBOR int if available
        if let Some(&oid) = self
            .registered_oid
            .table()
            .get_map()
            .get_by_right(self.registered_oid().c509_oid().oid())
        {
            encode_helper(e, "Attribute as OID int", ctx, &oid)?;
        } else {
            // Encode unwrapped CBOR OID
            self.registered_oid().c509_oid().encode(e, ctx)?;
        }

        // Check if the attribute value is empty
        if self.value.is_empty() {
            return Err(minicbor::encode::Error::message("Attribute value is empty"));
        }

        // If multi-value attributes, encode it as array
        if self.multi_value {
            encode_array_len(e, "Attribute multiple value", self.value.len() as u64)?;
        }

        // Encode each value in the attribute
        for value in &self.value {
            value.encode(e, ctx)?;
        }

        Ok(())
    }
}

impl Decode<'_, ()> for Attribute {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        // Handle CBOR int
        let mut attr = if decode_datatype(d, "Attribute as OID int")? == minicbor::data::Type::U8 {
            let i = decode_helper(d, "Attribute as OID int", ctx)?;
            let oid = get_oid_from_int(i).map_err(minicbor::decode::Error::message)?;
            Attribute::new(oid.clone())
        } else {
            // Handle unwrapped CBOR OID
            let c509_oid: C509oid = d.decode()?;
            Attribute::new(c509_oid.oid().clone())
        };

        // Handle attribute value
        if decode_datatype(d, "Attribute")? == minicbor::data::Type::Array {
            // When multi-value attribute
            let len = decode_array_len(d, "Attribute multiple value")?;

            if len == 0 {
                return Err(minicbor::decode::Error::message("Attribute value is empty"));
            }

            for _ in 0..len {
                attr.add_value(AttributeValue::decode(d, ctx)?);
            }
            attr = attr.set_multi_value();
        } else {
            let value = AttributeValue::decode(d, ctx)?;
            attr.add_value(value);
        }
        Ok(attr)
    }
}

// ------------------AttributeValue----------------------

/// An enum of possible value types for `Attribute`.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeValue {
    /// A text string.
    Text(String),
    /// A byte vector.
    Bytes(Vec<u8>),
}

impl Encode<()> for AttributeValue {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            AttributeValue::Text(text) => encode_helper(e, "Attribute value", ctx, text)?,
            AttributeValue::Bytes(bytes) => encode_helper(e, "Attribute value", ctx, bytes)?,
        };
        Ok(())
    }
}

impl Decode<'_, ()> for AttributeValue {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        match decode_datatype(d, "Attribute value")? {
            minicbor::data::Type::String => {
                Ok(AttributeValue::Text(decode_helper(
                    d,
                    "Attribute value",
                    ctx,
                )?))
            },
            minicbor::data::Type::Bytes => {
                Ok(AttributeValue::Bytes(decode_helper(
                    d,
                    "Attribute value",
                    ctx,
                )?))
            },
            _ => {
                Err(minicbor::decode::Error::message(
                    "Invalid AttributeValue, value should be either String or Bytes",
                ))
            },
        }
    }
}

// ------------------Test----------------------

#[cfg(test)]
mod test_attribute {
    use asn1_rs::oid;

    use super::*;

    #[test]
    fn encode_decode_attribute_int() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        let mut attribute = Attribute::new(oid!(1.2.840 .113549 .1 .9 .1));
        attribute.add_value(AttributeValue::Text("example@example.com".to_string()));
        attribute
            .encode(&mut encoder, &mut ())
            .expect("Failed to encode Attribute");
        // 1.2.840 .113549 .1 .9 .1 in attribute int = 0x00
        // Email Address example@example.com: 0x736578616d706c65406578616d706c652e636f6d
        assert_eq!(
            hex::encode(buffer.clone()),
            "00736578616d706c65406578616d706c652e636f6d"
        );

        let mut decoder = Decoder::new(&buffer);
        let attribute_decoded =
            Attribute::decode(&mut decoder, &mut ()).expect("Failed to decode Attribute");
        assert_eq!(attribute_decoded, attribute);
    }

    #[test]
    fn empty_attribute_value() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        let attribute = Attribute::new(oid!(1.2.840 .113549 .1 .9 .1));
        attribute
            .encode(&mut encoder, &mut ())
            .expect_err("Failed to encode Attribute");
    }
}
