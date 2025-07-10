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

impl minicbor::Encode<()> for Collaborators {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if !self.0.is_empty() {
            e.array(
                self.0
                    .len()
                    .try_into()
                    .map_err(minicbor::encode::Error::message)?,
            )?;
            for c in &self.0 {
                e.bytes(&c.to_string().into_bytes())?;
            }
        }
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Collaborators {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Array::decode(d, &mut DecodeCtx::Deterministic)?
            .iter()
            .map(|item| minicbor::Decoder::new(item).bytes())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(CatalystId::try_from)
            .collect::<Result<_, _>>()
            .map(Self)
            .map_err(minicbor::decode::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for Collaborators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|id| CatalystId::from_str(&id))
            .collect::<Result<_, _>>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
