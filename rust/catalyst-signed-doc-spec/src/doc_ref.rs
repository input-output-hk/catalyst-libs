//! `signed_doc.json` "ref" field JSON definition

use crate::{DocTypes, IsRequired};

/// `signed_doc.json` "ref" field JSON object
#[derive(serde::Deserialize)]
pub struct Ref {
    pub required: IsRequired,
    #[serde(rename = "type")]
    pub doc_type: DocTypes,
    pub multiple: Option<bool>,
}
