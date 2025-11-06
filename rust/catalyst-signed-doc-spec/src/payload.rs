//! `signed_doc.json` "payload" field JSON definition

/// `signed_doc.json` "payload" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Payload {
    pub nil: bool,
    pub schema: Option<Schema>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum Schema {
    Cddl(String),
    JsonSchema(serde_json::Value),
}
