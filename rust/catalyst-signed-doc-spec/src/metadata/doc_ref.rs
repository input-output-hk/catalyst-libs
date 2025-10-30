//! `signed_doc.json` "ref" field JSON definition

use crate::{doc_types::DocTypes, is_required::IsRequired};

/// `signed_doc.json` "ref" field JSON object
#[derive(serde::Deserialize)]
pub struct Ref {
    pub required: IsRequired,
    #[serde(rename = "type")]
    pub doc_type: DocTypes,
    #[serde(default)]
    pub multiple: bool,
}
