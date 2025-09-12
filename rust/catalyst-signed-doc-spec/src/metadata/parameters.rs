//! `signed_doc.json` "parameters" field JSON definition

use std::ops::Deref;

use crate::metadata::doc_ref::Ref;

/// `signed_doc.json` "parameters" field JSON object
#[derive(serde::Deserialize)]
pub struct Parameters(pub Ref);

impl Deref for Parameters {
    type Target = Ref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
