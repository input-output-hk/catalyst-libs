//! C509 Algorithm Identifier
//!
//! This module handle the `AlgorithmIdentifier` type where OID does not fall into the
//! table.
//!
//! ```cddl
//!    AlgorithmIdentifier = int / ~oid / [ algorithm: ~oid, parameters: bytes ]
//! ```
//!
//! **Note** `AlgorithmIdentifier` that have the same OID with different parameters are
//! not implemented yet.
//!
//! For more information about `AlgorithmIdentifier`,
//! visit [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/)

use asn1_rs::Oid;
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

use crate::oid::C509oid;

/// A struct represents the `AlgorithmIdentifier` type.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AlgorithmIdentifier {
    /// A `C509oid`
    c509_oid: C509oid,
    /// An optional parameter string
    param: Option<String>,
}

impl AlgorithmIdentifier {
    /// Create new instance of `AlgorithmIdentifier`.
    #[must_use]
    pub fn new(oid: Oid<'static>, param: Option<String>) -> Self {
        Self {
            c509_oid: C509oid::new(oid),
            param,
        }
    }

    /// Get the OID.
    #[must_use]
    pub fn oid(&self) -> &Oid<'static> {
        self.c509_oid.oid()
    }

    /// Get the parameter.
    #[must_use]
    pub fn param(&self) -> &Option<String> {
        &self.param
    }
}

impl Encode<()> for AlgorithmIdentifier {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match &self.param {
            // [ algorithm: ~oid, parameters: bytes ]
            Some(p) => {
                e.array(2)?;
                self.c509_oid.encode(e, ctx)?;
                e.bytes(p.as_bytes())?;
            },
            // ~oid
            None => {
                self.c509_oid.encode(e, ctx)?;
            },
        }
        Ok(())
    }
}

impl Decode<'_, ()> for AlgorithmIdentifier {
    fn decode(d: &mut Decoder<'_>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        // [ algorithm: ~oid, parameters: bytes ]
        if d.datatype()? == minicbor::data::Type::Array {
            let len = d.array()?.ok_or(minicbor::decode::Error::message(
                "Failed to get array length",
            ))?;
            if len != 2 {
                return Err(minicbor::decode::Error::message("Array length must be 2"));
            }
            let c509_oid = C509oid::decode(d, ctx)?;
            let param =
                String::from_utf8(d.bytes()?.to_vec()).map_err(minicbor::decode::Error::message)?;
            Ok(AlgorithmIdentifier::new(
                c509_oid.oid().clone(),
                Some(param),
            ))
            // ~oid
        } else {
            let oid = C509oid::decode(d, ctx)?;
            Ok(AlgorithmIdentifier::new(oid.oid().clone(), None))
        }
    }
}
