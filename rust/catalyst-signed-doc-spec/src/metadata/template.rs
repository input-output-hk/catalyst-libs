//! `signed_doc.json` "template" field JSON definition

use crate::{is_required::IsRequired, DocumentName};

/// `signed_doc.json` "template" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items, dead_code)]
pub struct Template {
    pub required: IsRequired,
    #[serde(rename = "type")]
    pub doc_type: Option<DocumentName>,
    pub multiple: Option<bool>,
}
