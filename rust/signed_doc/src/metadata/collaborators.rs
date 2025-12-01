//! Catalyst Signed Document `collaborators` field type definition.

use std::{ops::Deref, str::FromStr};

use catalyst_types::catalyst_id::CatalystId;
use cbork_utils::{array::Array, decode_context::DecodeCtx};

/// 'collaborators' field type definition, which is a JSON path string
#[derive(Clone, Debug, PartialEq)]
pub struct Collaborators(Vec<CatalystId>);

impl Deref for Collaborators {
    type Target = Vec<CatalystId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<CatalystId>> for Collaborators {
    fn from(value: Vec<CatalystId>) -> Self {
        Self(value)
    }
}

impl minicbor::Encode<()> for Collaborators {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(
            self.0
                .len()
                .try_into()
                .map_err(minicbor::encode::Error::message)?,
        )?;
        for c in &self.0 {
            e.bytes(&c.to_string().into_bytes())?;
        }
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Collaborators {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Array::decode(d, &mut DecodeCtx::ArrayDeterministic)
            .and_then(|arr| {
                if arr.is_empty() {
                    Err(minicbor::decode::Error::message(
                        "collaborators array must have at least one element",
                    ))
                } else {
                    Ok(arr)
                }
            })?
            .iter()
            .map(|item| minicbor::Decoder::new(item).bytes())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|id| {
                CatalystId::try_from(id)
                    .map_err(minicbor::decode::Error::custom)
                    .and_then(|id| {
                        if id.is_uri() {
                            Ok(id)
                        } else {
                            Err(minicbor::decode::Error::message(format!(
                            "provided CatalystId {id} must in URI format for collaborators field"
                        )))
                        }
                    })
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl<'de> serde::Deserialize<'de> for Collaborators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|id| {
                CatalystId::from_str(&id)
                    .map_err(serde::de::Error::custom)
                    .and_then(|id| {
                        if id.is_uri() {
                            Ok(id)
                        } else {
                            Err(serde::de::Error::custom(format!(
                                "provided CatalystId {id} must in ID format for collaborators field"
                            )))
                        }
                    })
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl serde::Serialize for Collaborators {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let iter = self.0.iter().map(ToString::to_string);
        serializer.collect_seq(iter)
    }
}

#[cfg(test)]
mod tests {
    use minicbor::{Decode, Decoder, Encode, Encoder};
    use test_case::test_case;

    use super::*;

    #[test_case(
        {
            Encoder::new(Vec::new())
        } ;
        "Invalid empty CBOR bytes"
    )]
    #[test_case(
        {
            let mut e = Encoder::new(Vec::new());
            e.array(0).unwrap();
            e
        } ;
        "Empty CBOR array"
    )]
    #[test_case(
        {
            let mut e = Encoder::new(Vec::new());
            e.array(1).unwrap();
            /* cspell:disable */
            e.bytes(b"preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3").unwrap();
            /* cspell:enable */
            e
        } ;
        "CatalystId not in ID form"
    )]
    fn test_invalid_cbor_decode(e: Encoder<Vec<u8>>) {
        assert!(
            Collaborators::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut ()).is_err()
        );
    }

    #[test]
    fn test_deterministic_decoding() {
        let mut cat_ids = vec![
            CatalystId::from_str(
                "id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3",
            )
            .unwrap(),
            CatalystId::from_str(
                "id.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1",
            )
            .unwrap(),
        ];
        cat_ids.sort_by(|a, b| {
            let a = a.to_string();
            let b = b.to_string();
            let a_bytes = a.as_bytes();
            let b_bytes = b.as_bytes();

            match a_bytes.len().cmp(&b_bytes.len()) {
                std::cmp::Ordering::Equal => a_bytes.cmp(b_bytes),
                other => other,
            }
        });

        let collabs = Collaborators::from(cat_ids.clone());
        let mut e = Encoder::new(Vec::new());
        collabs.encode(&mut e, &mut ()).unwrap();

        let result = Collaborators::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut ());
        assert!(result.is_ok());

        let mut e = Encoder::new(Vec::new());
        cat_ids.reverse();
        let collabs = Collaborators::from(cat_ids.clone());
        collabs.encode(&mut e, &mut ()).unwrap();

        let result = Collaborators::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut ());
        assert!(result.is_err());
    }
}
