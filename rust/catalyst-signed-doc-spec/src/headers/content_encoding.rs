//! `signed_doc.json` headers content encoding field JSON definition
//! Content Encoding

use crate::is_required::IsRequired;

#[derive(serde::Deserialize)]
pub struct ContentEncoding {
    pub required: IsRequired,
    pub value: Option<Vec<String>>,
}
