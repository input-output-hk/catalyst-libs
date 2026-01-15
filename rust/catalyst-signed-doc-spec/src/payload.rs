//! `signed_doc.json` "payload" field JSON definition

use catalyst_types::json_schema::JsonSchema;
use serde::Deserialize;

use crate::cddl_definitions::CddlType;

/// `signed_doc.json` "payload" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Payload {
    pub nil: bool,
    pub schema: Option<Schema>,
}

pub enum Schema {
    Cddl(CddlType),
    Json(serde_json::Value),
}

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        pub enum SchemaSerde {
            Cddl(CddlType),
            Json(serde_json::Value),
        }

        match SchemaSerde::deserialize(deserializer)? {
            SchemaSerde::Json(json) => {
                JsonSchema::try_from(&json)
                    .map(|_| Self::Json(json))
                    .map_err(serde::de::Error::custom)
            },
            SchemaSerde::Cddl(cddl_type) => Ok(Self::Cddl(cddl_type)),
        }
    }
}
