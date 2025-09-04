//! `signed_doc.json` "template" field JSON definition

use crate::signed_doc_spec::{DocumentName, IsRequired};

/// `signed_doc.json` "template" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items, dead_code)]
pub(crate) struct Template {
    pub(crate) required: IsRequired,
    #[serde(rename = "type")]
    pub(crate) doc_type: Option<DocumentName>,
    pub(crate) multiple: Option<bool>,
}
