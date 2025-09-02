//! `signed_doc.json` "ref" field JSON defintion

use crate::signed_doc_spec::{DocTypes, IsRequired};

/// `signed_doc.json` "ref" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Ref {
    pub(crate) required: IsRequired,
    #[serde(rename = "type")]
    pub(crate) doc_type: DocTypes,
    pub(crate) multiple: Option<bool>,
}
