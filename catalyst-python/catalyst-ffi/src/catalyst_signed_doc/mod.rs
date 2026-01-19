mod builder;

use catalyst_signed_doc_lib;

use crate::{Error, Result};

#[derive(uniffi::Object)]
struct CatalystSignedDocument(catalyst_signed_doc_lib::CatalystSignedDocument);

#[uniffi::export]
impl CatalystSignedDocument {
    fn id(&self) -> Result<String> {
        self.0
            .doc_id()
            .map(|v| v.to_string())
            .map_err(Error::Anyhow)
    }

    fn ver(&self) -> Result<String> {
        self.0
            .doc_ver()
            .map(|v| v.to_string())
            .map_err(Error::Anyhow)
    }

    fn r#type(&self) -> Result<String> {
        self.0
            .doc_type()
            .map(|v| v.to_string())
            .map_err(Error::Anyhow)
    }
}
