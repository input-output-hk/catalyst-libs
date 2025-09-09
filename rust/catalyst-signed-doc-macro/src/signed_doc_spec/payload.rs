//! `signed_doc.json` "payload" field JSON definition

/// `signed_doc.json` "template" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Payload {
    pub(crate) nil: bool,
    pub(crate) schema: Option<serde_json::Value>,
}
