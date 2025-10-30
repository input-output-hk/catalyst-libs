//! `signed_doc.json` "chain" field JSON definition

use crate::is_required::IsRequired;

/// `signed_doc.json` "chain" field JSON object
#[derive(serde::Deserialize)]
pub struct Chain {
    pub required: IsRequired,
}
