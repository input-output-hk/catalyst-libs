//! C509 General Names
//!
//! For more information about `GeneralNames`,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)

mod data;
pub mod general_name;
pub mod other_name_hw_module;
use general_name::GeneralName;
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

use crate::helper::{decode::decode_array_len, encode::encode_array_len};

/// A struct represents an array of `GeneralName`.
///
/// ```cddl
/// GeneralNames = [ + GeneralName ]
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GeneralNames(Vec<GeneralName>);

impl GeneralNames {
    /// Create a new instance of `GeneralNames` as empty vector.
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Get the inner of `GeneralName`.
    #[must_use]
    pub fn general_names(&self) -> &[GeneralName] {
        &self.0
    }

    /// Add a new `GeneralName` to the `GeneralNames`.
    pub fn add_general_name(&mut self, gn: GeneralName) {
        self.0.push(gn);
    }
}

impl Default for GeneralNames {
    fn default() -> Self {
        Self::new()
    }
}

impl Encode<()> for GeneralNames {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.0.is_empty() {
            return Err(minicbor::encode::Error::message(
                "General Names should not be empty",
            ));
        }
        // The general name type should be included in array too
        encode_array_len(e, "General Names", self.0.len() as u64 * 2)?;
        for gn in &self.0 {
            gn.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for GeneralNames {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "General Names")?;
        let mut gn = GeneralNames::new();
        for _ in 0..len / 2 {
            gn.add_general_name(GeneralName::decode(d, ctx)?);
        }
        Ok(gn)
    }
}

// ------------------Test----------------------

#[cfg(test)]
mod test_general_names {

    use std::net::Ipv4Addr;

    use asn1_rs::oid;
    use general_name::{GeneralNameTypeRegistry, GeneralNameValue};
    use other_name_hw_module::OtherNameHardwareModuleName;

    use super::*;
    use crate::oid::C509oid;

    #[test]
    fn encode_decode_gns() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut gns = GeneralNames::new();
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::OtherNameHardwareModuleName,
            GeneralNameValue::OtherNameHWModuleName(OtherNameHardwareModuleName::new(
                oid!(2.16.840 .1 .101 .3 .4 .2 .1),
                vec![0x01, 0x02, 0x03, 0x04],
            )),
        ));
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::IPAddress,
            GeneralNameValue::Bytes(Ipv4Addr::new(192, 168, 1, 1).octets().to_vec()),
        ));
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::RegisteredID,
            GeneralNameValue::Oid(C509oid::new(oid!(2.16.840 .1 .101 .3 .4 .2 .1))),
        ));
        gns.encode(&mut encoder, &mut ())
            .expect("Failed to encode GeneralNames");
        // Array of 4 GeneralName (type, value) so 8 items: 0x88
        assert_eq!(hex::encode(buffer.clone()), "88026b6578616d706c652e636f6d20824960864801650304020144010203040744c0a801010849608648016503040201");

        let mut decoder = Decoder::new(&buffer);
        let gns_decoded =
            GeneralNames::decode(&mut decoder, &mut ()).expect("Failed to decode GeneralName");
        assert_eq!(gns_decoded, gns);
    }

    #[test]
    fn encode_decode_gns_with_same_gn_type() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let mut gns = GeneralNames::new();
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::DNSName,
            GeneralNameValue::Text("example.com".to_string()),
        ));
        gns.encode(&mut encoder, &mut ())
            .expect("Failed to encode GeneralNames");
        // Array of 3 GeneralName (type, value) so 6 items: 0x86
        // DNSName with "example.com": 0x026b6578616d706c652e636f6d
        assert_eq!(
            hex::encode(buffer.clone()),
            "86026b6578616d706c652e636f6d026b6578616d706c652e636f6d026b6578616d706c652e636f6d"
        );

        let mut decoder = Decoder::new(&buffer);
        let gns_decoded =
            GeneralNames::decode(&mut decoder, &mut ()).expect("Failed to decode GeneralName");
        assert_eq!(gns_decoded, gns);
    }

    #[test]
    fn encode_decode_gns_empty() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let gns = GeneralNames::new();
        gns.encode(&mut encoder, &mut ())
            .expect_err("GeneralNames should not be empty");
    }
}
