//! `signed_doc.json` "collaborators" field JSON definition

use crate::is_required::IsRequired;

/// `signed_doc.json` "collaborators" field JSON object
#[derive(serde::Deserialize)]
pub struct Collaborators {
    pub required: IsRequired,
}
