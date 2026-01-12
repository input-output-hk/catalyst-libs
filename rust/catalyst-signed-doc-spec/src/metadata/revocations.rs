//! `signed_doc.json` "revocations" field JSON definition

use crate::is_required::IsRequired;

/// `signed_doc.json` "revocations" field JSON object
#[derive(serde::Deserialize)]
pub struct Revocations {
    pub required: IsRequired,
}
