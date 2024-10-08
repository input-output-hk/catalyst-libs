//! C509 Issuer Signature Algorithm
//! Certificate.
//!
//! ```cddl
//! subjectPublicKeyAlgorithm: AlgorithmIdentifier
//! ```
//!
//! For more information about `subjectPublicKeyAlgorithm`,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)
// cspell: words spka

mod data;

use std::str::FromStr;

use asn1_rs::Oid;
use data::{get_oid_from_int, SUBJECT_PUB_KEY_ALGO_LOOKUP};
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

/// A struct represents the `SubjectPubKeyAlgorithm`
#[derive(Debug, Clone, PartialEq)]
pub struct SubjectPubKeyAlgorithm {
    /// The registered OID of the `SubjectPubKeyAlgorithm`.
    registered_oid: C509oidRegistered,
    /// An `AlgorithmIdentifier` type
    algo_identifier: AlgorithmIdentifier,
}

impl SubjectPubKeyAlgorithm {
    /// Create new instance of `SubjectPubKeyAlgorithm` where it registered with
    /// Subject Public Key Algorithm lookup table.
    pub fn new(oid: Oid<'static>, param: Option<String>) -> Self {
        Self {
            registered_oid: C509oidRegistered::new(
                oid.clone(),
                SUBJECT_PUB_KEY_ALGO_LOOKUP.get_int_to_oid_table(),
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

/// Helper struct for deserialize and serialize `SubjectPubKeyAlgorithm`.
#[derive(Debug, Deserialize, Serialize)]
struct Helper {
    /// OID as string.
    oid: String,
    /// Optional parameter.
    param: Option<String>,
}

impl<'de> Deserialize<'de> for SubjectPubKeyAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let helper = Helper::deserialize(deserializer)?;
        let oid =
            Oid::from_str(&helper.oid).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;

        Ok(SubjectPubKeyAlgorithm::new(oid, helper.param))
    }
}

impl Serialize for SubjectPubKeyAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let helper = Helper {
            oid: self.registered_oid.c509_oid().oid().to_string(),
            param: self.algo_identifier.param().clone(),
        };
        helper.serialize(serializer)
    }
}

impl Encode<()> for SubjectPubKeyAlgorithm {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if let Some(&i) = self
            .registered_oid
            .table()
            .get_map()
            .get_by_right(self.registered_oid.c509_oid().oid())
        {
            encode_helper(e, "Subject public key algorithm", ctx, &i)?;
        } else {
            AlgorithmIdentifier::encode(&self.algo_identifier, e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for SubjectPubKeyAlgorithm {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        // Check u8 for 0 - 28
        if decode_datatype(d, "Subject public key algorithm")? == minicbor::data::Type::U8 {
            let i = decode_helper(d, "Subject public key algorithm", ctx)?;
            let oid = get_oid_from_int(i).map_err(minicbor::decode::Error::message)?;
            Ok(Self::new(oid, None))
        } else {
            let algo_identifier = AlgorithmIdentifier::decode(d, ctx)?;
            Ok(SubjectPubKeyAlgorithm::new(
                algo_identifier.oid().clone(),
                algo_identifier.param().clone(),
            ))
        }
    }
}

// ------------------Test----------------------

#[cfg(test)]
mod test_subject_public_key_algorithm {
    use asn1_rs::oid;

    use super::*;

    #[test]
    fn test_registered_oid() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let spka = SubjectPubKeyAlgorithm::new(oid!(1.3.101 .112), None);
        spka.encode(&mut encoder, &mut ())
            .expect("Failed to encode SubjectPubKeyAlgorithm");

        // Ed25519 - int 10: 0x0a
        assert_eq!(hex::encode(buffer.clone()), "0a");

        let mut decoder = Decoder::new(&buffer);
        let decoded_spka = SubjectPubKeyAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode SubjectPubKeyAlgorithm");
        assert_eq!(decoded_spka, spka);
    }

    #[test]
    fn test_unregistered_oid() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let spka = SubjectPubKeyAlgorithm::new(oid!(2.16.840 .1 .101 .3 .4 .2 .1), None);
        spka.encode(&mut encoder, &mut ())
            .expect("Failed to encode SubjectPubKeyAlgorithm");

        // 2.16.840 .1 .101 .3 .4 .2 .1: 0x49608648016503040201
        assert_eq!(hex::encode(buffer.clone()), "49608648016503040201");

        let mut decoder = Decoder::new(&buffer);
        let decoded_spka = SubjectPubKeyAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode SubjectPubKeyAlgorithm");
        assert_eq!(decoded_spka, spka);
    }

    #[test]
    fn test_unregistered_oid_with_param() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let spka = SubjectPubKeyAlgorithm::new(
            oid!(2.16.840 .1 .101 .3 .4 .2 .1),
            Some("example".to_string()),
        );
        spka.encode(&mut encoder, &mut ())
            .expect("Failed to encode SubjectPubKeyAlgorithm");
        // Array of 2 items: 0x82
        // 2.16.840 .1 .101 .3 .4 .2 .1: 0x49608648016503040201
        // bytes "example": 0x476578616d706c65
        assert_eq!(
            hex::encode(buffer.clone()),
            "8249608648016503040201476578616d706c65"
        );

        let mut decoder = Decoder::new(&buffer);
        let decoded_spka = SubjectPubKeyAlgorithm::decode(&mut decoder, &mut ())
            .expect("Failed to decode SubjectPubKeyAlgorithm");
        assert_eq!(decoded_spka, spka);
    }
}
