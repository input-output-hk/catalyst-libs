//! Catalyst Signed Document `section` field type definition.

use std::fmt::{Debug, Display};

use catalyst_types::uuid::{CborContext, UuidV7};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
use minicbor::{Decode, Decoder, Encode, Encoder, data::Type, encode::Write};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// A list of all versions of this document which are 'revoked'.
#[derive(Clone, Debug, PartialEq)]
pub enum Revocations {
    /// All documents are affected.
    All,
    /// A specified list.
    Specified(Vec<UuidV7>),
}

impl From<Vec<UuidV7>> for Revocations {
    fn from(value: Vec<UuidV7>) -> Self {
        Self::Specified(value)
    }
}

impl Serialize for Revocations {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Revocations::All => serializer.serialize_bool(true),
            Revocations::Specified(versions) => versions.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Revocations {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = serde_json::Value::deserialize(deserializer)?;

        match value {
            serde_json::Value::Bool(true) => Ok(Revocations::All),

            serde_json::Value::Array(_) => {
                let versions = Vec::<UuidV7>::deserialize(value).map_err(D::Error::custom)?;
                Ok(Revocations::Specified(versions))
            },

            _ => {
                Err(D::Error::custom(
                    "invalid revocations value: expected `true` or array of UUIDv7",
                ))
            },
        }
    }
}

impl Display for Revocations {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Specified(versions) => write!(f, "{versions:?}"),
        }
    }
}

impl Encode<()> for Revocations {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Revocations::All => {
                e.bool(true)?;
            },
            Revocations::Specified(versions) => {
                versions.encode(e, &mut CborContext::Tagged)?;
            },
        }
        Ok(())
    }
}

impl<'b> Decode<'b, ()> for Revocations {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "Revocations decoding";

        match d.datatype()? {
            Type::Bool => {
                if d.bool()? {
                    Ok(Revocations::All)
                } else {
                    Err(minicbor::decode::Error::message(
                        "{CONTEXT}: `false` value is not allowed",
                    ))
                }
            },
            Type::Array => {
                let versions = Array::decode(d, &mut DecodeCtx::ArrayDeterministic)
                    .map_err(|e| minicbor::decode::Error::message(format!("{CONTEXT}: {e}")))?
                    .into_iter()
                    .map(|ver_bytes| {
                        UuidV7::decode(
                            &mut minicbor::Decoder::new(ver_bytes.as_slice()),
                            &mut CborContext::Tagged,
                        )
                        .map_err(|e| e.with_message("Invalid Ver UUIDv7"))
                    })
                    .collect::<Result<_, _>>()?;

                Ok(Revocations::Specified(versions))
            },
            _ => {
                Err(minicbor::decode::Error::message(
                    "{CONTEXT}: expected bool(true) or array",
                ))
            },
        }
    }
}
