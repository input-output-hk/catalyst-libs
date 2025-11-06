//! `signed_doc.json` "payload" field JSON definition

use catalyst_types::json_schema::JsonSchema;
use serde::Deserialize;

/// `signed_doc.json` "payload" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Payload {
    pub nil: bool,
    pub schema: Option<Schema>,
}

pub enum Schema {
    Cddl(String),
    Json(JsonSchema),
}

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        pub enum SchemaSerde {
            Cddl(String),
            Json(serde_json::Value),
        }

        match SchemaSerde::deserialize(deserializer)? {
            SchemaSerde::Json(schema) => {
                JsonSchema::try_from(&schema)
                    .map(|v| Self::Json(v))
                    .map_err(|e| serde::de::Error::custom(e))
            },
            SchemaSerde::Cddl(schema) => Ok(Self::Cddl(schema)),
        }
    }
}
