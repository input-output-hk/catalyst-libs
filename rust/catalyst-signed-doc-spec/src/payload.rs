//! `signed_doc.json` "payload" field JSON definition

/// `signed_doc.json` "payload" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Payload {
    pub nil: bool,
    pub schema: Option<serde_json::Value>,
}
