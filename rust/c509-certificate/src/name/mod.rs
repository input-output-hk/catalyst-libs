//! C509 type Name
//!
//! Currently only support natively signed c509 certificate, so all text strings
//! are UTF-8 encoded and all attributeType should be positive.
//!
//! ```cddl
//! Name = [ * Attribute ] / text / bytes
//! Attribute = ( attributeType: int, attributeValue: text ) //
//!             ( attributeType: ~oid, attributeValue: bytes ) //
//! ```
//!
//! For more information about Name,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)

use asn1_rs::{oid, Oid};
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    attributes::attribute::{Attribute, AttributeValue},
    helper::{
        decode::{decode_array_len, decode_bytes, decode_datatype, decode_helper},
        encode::{encode_array_len, encode_bytes, encode_helper},
    },
};
/// OID of `CommonName` attribute.
const COMMON_NAME_OID: Oid<'static> = oid!(2.5.4 .3);
/// EUI-64 prefix.
const EUI64_PREFIX: u8 = 0x01;
/// Hex prefix.
const HEX_PREFIX: u8 = 0x00;
/// Total length of CBOR byte for EUI-64.
const EUI64_LEN: usize = 9;
/// Total length of CBOR byte for EUI-64 mapped from a 48-bit MAC address.
const EUI64_MAC_LEN: usize = 7;

// ------------------Name----------------------

/// A struct of C509 Name with `NameValue`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Name(NameValue);

impl Name {
    /// Create a new instance of `Name` its value.
    #[must_use]
    pub fn new(value: NameValue) -> Self {
        Self(value)
    }

    /// Get the value of the `Name`.
    #[must_use]
    pub fn value(&self) -> &NameValue {
        &self.0
    }
}

impl Encode<()> for Name {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for Name {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        NameValue::decode(d, ctx).map(Name::new)
    }
}

// ------------------NameValue----------------------

/// An enum of possible value types for `Name`.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameValue {
    /// Attribute.
    Attribute(Vec<Attribute>),
    /// A text.
    Text(String),
    /// bytes.
    Bytes(Vec<u8>),
}

impl Encode<()> for NameValue {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            NameValue::Attribute(attrs) => {
                if let Some(attr_first) = attrs.first() {
                    // If `attrs` contains exactly one attribute of type CommonName
                    if attrs.len() == 1
                        && attr_first.registered_oid().c509_oid().oid() == &COMMON_NAME_OID
                    {
                        // Get the value of the attribute
                        let cn_value =
                            attr_first
                                .value()
                                .first()
                                .ok_or(minicbor::encode::Error::message(
                                    "Cannot get the first attribute value",
                                ))?;

                        encode_cn_value(e, cn_value)?;
                    } else {
                        let len = (attrs.len() as u64).checked_mul(2).ok_or_else(|| {
                            minicbor::encode::Error::message("Attribute length overflow")
                        })?;
                        encode_array_len(e, "Attributes", len)?;
                        for attribute in attrs {
                            attribute.encode(e, ctx)?;
                        }
                    }
                } else {
                    // If is okay if the attributes is empty
                    let len = (attrs.len() as u64).checked_mul(2).ok_or_else(|| {
                        minicbor::encode::Error::message("Attribute length overflow")
                    })?;
                    encode_array_len(e, "Attributes", len)?;
                    for attribute in attrs {
                        attribute.encode(e, ctx)?;
                    }
                }
            },
            NameValue::Text(text) => {
                encode_helper(e, "Name", ctx, text)?;
            },
            NameValue::Bytes(bytes) => {
                encode_bytes(e, "Name", bytes)?;
            },
        }
        Ok(())
    }
}

impl Decode<'_, ()> for NameValue {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        match decode_datatype(d, "Name")? {
            minicbor::data::Type::Array => {
                let len = decode_array_len(d, "Attributes")?;
                let mut attrs = Vec::new();

                // The attribute type is included in an array, so divide by 2
                for _ in 0..len / 2 {
                    let attribute = Attribute::decode(d, ctx)?;
                    attrs.push(attribute);
                }
                Ok(NameValue::Attribute(attrs))
            },
            // If Name is a text string, the attribute is a CommonName
            minicbor::data::Type::String => {
                Ok(create_attributes_with_cn(decode_helper(d, "Name", ctx)?))
            },
            minicbor::data::Type::Bytes => decode_bytes_helper(d),
            _ => {
                Err(minicbor::decode::Error::message(
                    "Name must be an array, text or bytes",
                ))
            },
        }
    }
}

/// Encode common name value.
fn encode_cn_value<W: Write>(
    e: &mut Encoder<W>, cn_value: &AttributeValue,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    let hex_regex = Regex::new(r"^[0-9a-f]+$").map_err(minicbor::encode::Error::message)?;
    let eui64_regex =
        Regex::new(r"^([0-9A-F]{2}-){7}[0-9A-F]{2}$").map_err(minicbor::encode::Error::message)?;
    let mac_eui64_regex = Regex::new(r"^([0-9A-F]{2}-){3}FF-FE-([0-9A-F]{2}-){2}[0-9A-F]{2}$")
        .map_err(minicbor::encode::Error::message)?;

    match cn_value {
        AttributeValue::Text(s) => {
            // If the text string has an even length ≥ 2 and contains only the
            // symbols '0'–'9' or 'a'–'f', it is encoded as a CBOR byte
            // string, prefixed with an initial byte set to '00'
            if hex_regex.is_match(s) && s.len() % 2 == 0 {
                let decoded_bytes = hex::decode(s).map_err(minicbor::encode::Error::message)?;
                encode_bytes(
                    e,
                    "Common Name hex",
                    &[&[HEX_PREFIX], &decoded_bytes[..]].concat(),
                )?;

            // An EUI-64 mapped from a 48-bit MAC address (i.e., of the form
            // "HH-HH-HH-FF-FE-HH-HH-HH) is encoded as a CBOR byte string prefixed with an
            // initial byte set to '01', for a total length of 7.
            } else if mac_eui64_regex.is_match(s) {
                let clean_name = s.replace('-', "");
                let decoded_bytes =
                    hex::decode(clean_name).map_err(minicbor::encode::Error::message)?;
                let chunk2 = decoded_bytes
                    .get(..3)
                    .ok_or(minicbor::encode::Error::message(
                        "Failed to get MAC EUI-64 bytes index 0 to 2",
                    ))?;
                let chunk3 = decoded_bytes
                    .get(5..)
                    .ok_or(minicbor::encode::Error::message(
                        "Failed to get MAC EUI-64 bytes index 5 to 6",
                    ))?;
                encode_bytes(
                    e,
                    "Common Name EUI-64 MAC",
                    &[&[EUI64_PREFIX], chunk2, chunk3].concat(),
                )?;

            // an EUI-64 of the form "HH-HH-HH-HH-HH-HH-HH-HH" where 'H'
            // is one of the symbols '0'–'9' or 'A'–'F' it is encoded as a
            // CBOR byte string prefixed with an initial byte set to '01', for a total
            // length of 9.
            } else if eui64_regex.is_match(s) {
                let clean_name = s.replace('-', "");
                let decoded_bytes =
                    hex::decode(clean_name).map_err(minicbor::encode::Error::message)?;
                encode_bytes(
                    e,
                    "Common Name EUI-64",
                    &[&[EUI64_PREFIX], &decoded_bytes[..]].concat(),
                )?;
            } else {
                encode_helper(e, "Common Name", &mut (), s)?;
            }
        },
        AttributeValue::Bytes(_) => {
            return Err(minicbor::encode::Error::message(
                "CommonName attribute value must be a text string",
            ));
        },
    }
    Ok(())
}

/// Format EUI bytes.
fn formatted_eui_bytes(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join("-")
}

/// Decode bytes.
fn decode_bytes_helper(d: &mut Decoder<'_>) -> Result<NameValue, minicbor::decode::Error> {
    let bytes = decode_bytes(d, "Name")?;

    let first_i = bytes.first().ok_or(minicbor::decode::Error::message(
        "Failed to get the first index of bytes",
    ))?;

    // Bytes prefix
    match *first_i {
        // 0x00 for hex
        HEX_PREFIX => decode_hex_cn_bytes(&bytes),
        // 0x01 for EUI
        EUI64_PREFIX => decode_eui_cn_bytes(&bytes),
        _ => Ok(NameValue::Bytes(bytes)),
    }
}

/// Decode common name hex bytes.
fn decode_hex_cn_bytes(bytes: &[u8]) -> Result<NameValue, minicbor::decode::Error> {
    let text = hex::encode(bytes.get(1..).ok_or(minicbor::decode::Error::message(
        "Failed to get hex bytes index",
    ))?);
    Ok(create_attributes_with_cn(text))
}

/// Decode common name EUI-64 bytes.
fn decode_eui_cn_bytes(bytes: &[u8]) -> Result<NameValue, minicbor::decode::Error> {
    // Check the length of the bytes to determine what EUI type it is
    match bytes.len() {
        // EUI-64 mapped from a 48-bit MAC address
        EUI64_MAC_LEN => {
            let chunk1 = bytes.get(1..4).ok_or(minicbor::decode::Error::message(
                "Failed to get EUI-64 bytes index 1 to 3",
            ))?;
            let chunk4 = bytes.get(4..).ok_or(minicbor::decode::Error::message(
                "Failed to get EUI-64 bytes index 4 to 7",
            ))?;
            // Turn it into HH-HH-HH-FF-FE-HH-HH-HH
            let data = [chunk1, &[0xFF], &[0xFE], chunk4].concat();
            let text = formatted_eui_bytes(&data);
            Ok(create_attributes_with_cn(text))
        },
        // EUI-64
        EUI64_LEN => {
            let text = formatted_eui_bytes(bytes.get(1..).ok_or(
                minicbor::decode::Error::message("Failed to get EUI-64 bytes index"),
            )?);
            Ok(create_attributes_with_cn(text))
        },
        _ => {
            Err(minicbor::decode::Error::message(
                "EUI-64 or MAC address must be 7 or 9 bytes",
            ))
        },
    }
}

/// Create a attributes with attribute common name from string.
fn create_attributes_with_cn(text: String) -> NameValue {
    let mut attr = Attribute::new(COMMON_NAME_OID);
    attr.add_value(AttributeValue::Text(text));
    NameValue::Attribute(vec![attr])
}

// ------------------Test----------------------

#[cfg(test)]
mod test_name {
    use std::vec;

    use super::*;
    use crate::attributes::attribute::Attribute;

    // Test data from https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
    // A.1.1.  Example C509 Certificate Encoding
    #[test]
    fn encode_decode_type_name_cn() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("RFC test CA".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));
        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // "RFC test CA" text(11): 0x6b5246432074657374204341
        assert_eq!(hex::encode(buffer.clone()), "6b5246432074657374204341");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_hex() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("000123abcd".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));
        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // Bytes of length 6: 0x46
        // Prefix of CommonName hex: 0x00
        // Bytes 000123abcd: 0x000123abcd
        assert_eq!(hex::encode(buffer.clone()), "4600000123abcd");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_hex_cap() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("000123ABCD".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));
        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // String of len 10: 0x6a
        // String 000123abcd: 30303031323341424344
        assert_eq!(hex::encode(buffer.clone()), "6a30303031323341424344");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_cn_eui_mac() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("01-23-45-FF-FE-67-89-AB".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");
        // Bytes of length 7: 0x47
        // "01-23-45-FF-FE-67-89-AB" special encode: 0x010123456789AB
        assert_eq!(hex::encode(buffer.clone()), "47010123456789ab");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_cn_eui_mac_un_cap() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("01-23-45-ff-fe-67-89-AB".to_string()));
        let name = Name::new(NameValue::Attribute(vec![attr]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // String of len 23: 0x77
        // "01-23-45-ff-fe-67-89-AB": 0x7730312d32332d34352d66662d66652d36372d38392d4142
        assert_eq!(
            hex::encode(buffer.clone()),
            "7730312d32332d34352d66662d66652d36372d38392d4142"
        );

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_cn_eui() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("01-23-45-67-89-AB-00-01".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // 01-23-45-67-89-AB-00-01 = h'010123456789AB0001': 0x49010123456789ab0001
        assert_eq!(hex::encode(buffer.clone()), "49010123456789ab0001");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_type_name_cn_eui_un_cap() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr = Attribute::new(oid!(2.5.4 .3));
        attr.add_value(AttributeValue::Text("01-23-45-67-89-ab-00-01".to_string()));

        let name = Name::new(NameValue::Attribute(vec![attr]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        // String of len 23: 0x77
        // "01-23-45-67-89-ab-00-01": 0x7730312d32332d34352d36372d38392d61622d30302d3031
        assert_eq!(
            hex::encode(buffer.clone()),
            "7730312d32332d34352d36372d38392d61622d30302d3031"
        );

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    // Test data from https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
    // A.2.  Example IEEE 802.1AR profiled X.509 Certificate
    // Issuer: C=US, ST=CA, O=Example Inc, OU=certification, CN=802.1AR CA
    #[test]
    fn encode_decode_type_name_attrs() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut attr1 = Attribute::new(oid!(2.5.4 .6));
        attr1.add_value(AttributeValue::Text("US".to_string()));
        let mut attr2 = Attribute::new(oid!(2.5.4 .8));
        attr2.add_value(AttributeValue::Text("CA".to_string()));
        let mut attr3 = Attribute::new(oid!(2.5.4 .10));
        attr3.add_value(AttributeValue::Text("Example Inc".to_string()));
        let mut attr4 = Attribute::new(oid!(2.5.4 .11));
        attr4.add_value(AttributeValue::Text("certification".to_string()));
        let mut attr5 = Attribute::new(oid!(2.5.4 .3));
        attr5.add_value(AttributeValue::Text("802.1AR CA".to_string()));

        let name = Name::new(NameValue::Attribute(vec![
            attr1, attr2, attr3, attr4, attr5,
        ]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");
        assert_eq!(hex::encode(buffer.clone()), "8a0462555306624341086b4578616d706c6520496e63096d63657274696669636174696f6e016a3830322e314152204341");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }

    #[test]
    fn encode_decode_empty_attribute() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let name = Name::new(NameValue::Attribute(vec![]));

        name.encode(&mut encoder, &mut ())
            .expect("Failed to encode Name");

        assert_eq!(hex::encode(buffer.clone()), "80");

        let mut decoder = Decoder::new(&buffer);
        let name_decoded = Name::decode(&mut decoder, &mut ()).expect("Failed to decode Name");
        assert_eq!(name_decoded, name);
    }
}
