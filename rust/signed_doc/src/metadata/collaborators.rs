//! Catalyst Signed Document `collabs` field type definition.

use std::ops::Deref;

/// 'collabs' field type definition, which is a JSON path string
#[derive(Clone, Debug, PartialEq)]
pub struct Collaborators(Vec<String>);

impl Deref for Collaborators {
    type Target = Vec<String>;

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
                e.str(c)?;
            }
        }
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Collaborators {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let Some(items) = d.array()? else {
            return Err(minicbor::decode::Error::message(
                "Must a definite size array",
            ));
        };
        let collabs = (0..items)
            .map(|_| Ok(d.str()?.to_string()))
            .collect::<Result<_, _>>()?;
        Ok(Self(collabs))
    }
}

impl<'de> serde::Deserialize<'de> for Collaborators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Ok(Self(Vec::<String>::deserialize(deserializer)?))
    }
}
