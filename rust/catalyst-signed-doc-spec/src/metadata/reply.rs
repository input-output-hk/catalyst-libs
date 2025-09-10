//! `signed_doc.json` "reply" field JSON definition

use std::ops::Deref;

use crate::metadata::doc_ref::Ref;

/// `signed_doc.json` "reply" field JSON object
#[derive(serde::Deserialize)]
pub struct Reply(pub Ref);

impl Deref for Reply {
    type Target = Ref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
