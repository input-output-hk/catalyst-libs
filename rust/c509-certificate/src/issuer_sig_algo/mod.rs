//! C509 Issuer Signature Algorithm
//! Certificate.
//!
//! ```cddl
//! issuerSignatureAlgorithm: AlgorithmIdentifier
//! ```
//!
//! For more information about `issuerSignatureAlgorithm`,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)

mod data;

use std::str::FromStr;

use asn1_rs::Oid;
use data::{get_oid_from_int, ISSUER_SIG_ALGO_LOOKUP};
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    algorithm_identifier::AlgorithmIdentifier,
    helper::{
        decode::{decode_datatype, decode_helper},
        encode::encode_helper,
    },
    oid::C509oidRegistered,
};

/// A struct represents the `IssuerSignatureAlgorithm`
#[derive(Debug, Clone, PartialEq)]
pub struct IssuerSignatureAlgorithm {
    /// The registered OID of the `IssuerSignatureAlgorithm`.
    registered_oid: C509oidRegistered,
    /// An `AlgorithmIdentifier` type
    algo_identifier: AlgorithmIdentifier,
}

impl IssuerSignatureAlgorithm {
    /// Create new instance of `IssuerSignatureAlgorithm` where it registered with
    /// Issuer Signature Algorithm lookup table.
    pub fn new(oid: Oid<'static>, param: Option<String>) -> Self {
        Self {
            registered_oid: C509oidRegistered::new(
                oid.clone(),
                ISSUER_SIG_ALGO_LOOKUP.get_int_to_oid_table(),
            ),
            algo_identifier: AlgorithmIdentifier::new(oid, param),
        }
    }

    /// Get the algorithm identifier.
    #[must_use]
    pub fn algo_identifier(&self) -> &AlgorithmIdentifier {
        &self.algo_identifier
    }

    /// Get the registered OID.
    #[allow(dead_code)]
    pub(crate) fn registered_oid(&self) -> &C509oidRegistered {
        &self.registered_oid
    }
}

/// Helper struct for deserialize and serialize `IssuerSignatureAlgorithm`.
#[derive(Debug, Deserialize, Serialize)]
struct Helper {
    /// OID as string.
    oid: String,
    /// Optional parameter.
    param: Option<String>,
}

impl<'de> Deserialize<'de> for IssuerSignatureAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let helper = Helper::deserialize(deserializer)?;
        let oid =
            Oid::from_str(&helper.oid).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;

        Ok(IssuerSignatureAlgorithm::new(oid, helper.param))
    }
}

impl Serialize for IssuerSignatureAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let helper = Helper {
            oid: self.registered_oid.c509_oid().oid().to_string(),
            param: self.algo_identifier.param().clone(),
        };
        helper.serialize(serializer)
    }
}

impl Encode<()> for IssuerSignatureAlgorithm {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if let Some(&i) = self
            .registered_oid
            .table()
            .get_map()
            .get_by_right(self.registered_oid.c509_oid().oid())
        {
            encode_helper(e, "Issuer Signature Algorithm as OID int", ctx, &i)?;
        } else {
            AlgorithmIdentifier::encode(&self.algo_identifier, e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for IssuerSignatureAlgorithm {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        match decode_datatype(d, "Issuer Signature Algorithm")? {
            // Check i16 for -256 and -256
            minicbor::data::Type::U8 | minicbor::data::Type::I16 => {
                let i = decode_helper(d, "Issuer Signature Algorithm as OID int", ctx)?;
                let oid = get_oid_from_int(i).map_err(minicbor::decode::Error::message)?;
                Ok(Self::new(oid, None))
            },
            _ => {
                let algo_identifier = AlgorithmIdentifier::decode(d, ctx)?;
                Ok(IssuerSignatureAlgorithm::new(
                    algo_identifier.oid().clone(),
                    algo_identifier.param().clone(),
                ))
            },
        }
    }
}

// ------------------Test----------------------

#[cfg(test)]
mod test_issuer_signature_algorithm {
    use asn1_rs::oid;

    use super::*;

    #[test]
    fn test_registered_oid() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let isa = IssuerSignatureAlgorithm::new(oid!(1.3.101 .112), None);
        isa.encode(&mut encoder, &mut ())
            .expect("Failed to encode IssuerSignatureAlgorithm");

        // Ed25519 - int 12: 0x0c
        assert_eq!(hex::encode(buffer.clone()), "0c");

        let mut decoder = Decoder::new(&buffer);
        let decoded_isa = IssuerSignatureAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode IssuerSignatureAlgorithm");
        assert_eq!(decoded_isa, isa);
    }

    #[test]
    fn test_unregistered_oid() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let isa = IssuerSignatureAlgorithm::new(oid!(2.16.840 .1 .101 .3 .4 .2 .1), None);
        isa.encode(&mut encoder, &mut ())
            .expect("Failed to encode IssuerSignatureAlgorithm");

        // 2.16.840 .1 .101 .3 .4 .2 .1: 0x49608648016503040201
        assert_eq!(hex::encode(buffer.clone()), "49608648016503040201");

        let mut decoder = Decoder::new(&buffer);
        let decoded_isa = IssuerSignatureAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode IssuerSignatureAlgorithm");
        assert_eq!(decoded_isa, isa);
    }

    #[test]
    fn test_unregistered_oid_with_param() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let isa = IssuerSignatureAlgorithm::new(
            oid!(2.16.840 .1 .101 .3 .4 .2 .1),
            Some("example".to_string()),
        );
        isa.encode(&mut encoder, &mut ())
            .expect("Failed to encode IssuerSignatureAlgorithm");
        // Array of 2 items: 0x82
        // 2.16.840 .1 .101 .3 .4 .2 .1: 0x49608648016503040201
        // bytes "example": 0x476578616d706c65
        assert_eq!(
            hex::encode(buffer.clone()),
            "8249608648016503040201476578616d706c65"
        );

        let mut decoder = Decoder::new(&buffer);
        let decoded_isa = IssuerSignatureAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode IssuerSignatureAlgorithm");
        assert_eq!(decoded_isa, isa);
    }
}
