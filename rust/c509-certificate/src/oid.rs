//! C509 OID provides an encoding and decoding of C509 Object Identifier (OID).
//!
//! Please refer to [RFC9090](https://datatracker.ietf.org/doc/rfc9090/) for OID encoding
//! Please refer to [CDDL Wrapping](https://datatracker.ietf.org/doc/html/rfc8610#section-3.7)
//! for unwrapped types.

use std::str::FromStr;

use anyhow::Result;
use minicbor::{Decode, Decoder, Encode, Encoder, decode, encode::Write};
use oid_registry::Oid;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    helper::{decode::decode_bytes, encode::encode_bytes},
    tables::IntegerToOidTable,
};

/// A strut of C509 OID with Registered Integer.
#[derive(Debug, Clone, PartialEq)]
pub struct C509oidRegistered {
    /// The `C509oid`.
    c509_oid: C509oid,
    /// The registration lookup table for associated int to OID.
    /// Each C509 field or type can have different registration table.
    registration_table: &'static IntegerToOidTable,
}

impl C509oidRegistered {
    /// Create a new instance of `C509oidRegistered`.
    pub(crate) fn new(
        oid: Oid<'static>,
        table: &'static IntegerToOidTable,
    ) -> Self {
        Self {
            c509_oid: C509oid::new(oid),
            registration_table: table,
        }
    }

    /// Get the `C509oid`.
    #[must_use]
    pub fn c509_oid(&self) -> &C509oid {
        &self.c509_oid
    }

    /// Get the registration table.
    pub(crate) fn table(&self) -> &'static IntegerToOidTable {
        self.registration_table
    }
}

// -----------------------------------------

/// A struct represent an instance of `C509oid`.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct C509oid(Oid<'static>);

/// A helper struct for deserialize and serialize `C509oid`.
#[derive(Debug, Deserialize, Serialize)]
struct Helper {
    /// OID value in string.
    oid: String,
}

impl C509oid {
    /// Create an new instance of `C509oid`.
    #[must_use]
    pub fn new(oid: Oid<'static>) -> Self {
        Self(oid)
    }

    /// Get the underlying OID of the `C509oid`
    #[must_use]
    pub fn oid(&self) -> &Oid<'static> {
        &self.0
    }
}

impl<'de> Deserialize<'de> for C509oid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let helper = Helper::deserialize(deserializer)?;
        let oid =
            Oid::from_str(&helper.oid).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(C509oid::new(oid))
    }
}

impl Serialize for C509oid {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let helper = Helper {
            oid: self.0.to_string(),
        };
        helper.serialize(serializer)
    }
}

impl Encode<()> for C509oid {
    /// Encode an OID
    /// Encode as an unwrapped OID (~oid) - as bytes string without tag.
    ///
    /// # Returns
    ///
    /// A vector of bytes containing the CBOR encoded OID.
    /// If the encoding fails, it will return an error.
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let oid_bytes = self.0.as_bytes();
        encode_bytes(e, "C509 OID", oid_bytes)
    }
}

impl Decode<'_, ()> for C509oid {
    /// Decode an OID
    /// Decode the OID as unwrapped OID (~oid) - as bytes string without tag.
    ///
    /// # Returns
    ///
    /// A C509oid instance.
    /// If the decoding fails, it will return an error.
    fn decode(
        d: &mut Decoder,
        _ctx: &mut (),
    ) -> Result<Self, decode::Error> {
        let oid_bytes = decode_bytes(d, "C509 OID")?;
        let oid = Oid::new(oid_bytes.into());
        Ok(C509oid::new(oid))
    }
}

// -----------------------------------------

#[cfg(test)]
mod test_c509_oid {

    use asn1_rs::oid;

    use super::*;

    // Test reference 3.1. Encoding of the SHA-256 OID
    // https://datatracker.ietf.org/doc/rfc9090/
    #[test]
    fn encode_decode_unwrapped() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        let oid = C509oid::new(oid!(2.16.840.1.101.3.4.2.1));
        oid.encode(&mut encoder, &mut ())
            .expect("Failed to encode OID");
        // bytes(9) 0x49
        // 0x60 (for 2.16)
        // 0x18, 0x03, 0x60, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01
        assert_eq!(hex::encode(buffer.clone()), "49608648016503040201");

        let mut decoder = Decoder::new(&buffer);
        let decoded_oid = C509oid::decode(&mut decoder, &mut ()).expect("Failed to decode OID");
        assert_eq!(decoded_oid, oid);
    }

    #[test]
    fn partial_equal() {
        let oid1 = C509oid::new(oid_registry::OID_HASH_SHA1);
        let oid2 = C509oid::new(oid!(1.3.14.3.2.26));
        assert_eq!(oid1, oid2);
    }
}
