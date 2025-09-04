//! `signed_doc.json` "payload" field JSON definition

use crate::signed_doc_spec::IsRequired;

/// `signed_doc.json` "template" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items, dead_code)]
pub(crate) struct Payload {
    pub(crate) required: IsRequired,
}
