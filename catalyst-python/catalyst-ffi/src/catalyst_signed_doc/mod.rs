mod builder;

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
            .map(ToString::to_string)
            .map_err(Error::Anyhow)
    }

    fn r#ref(&self) -> Result<DocumentRef> {
        self.0.doc_ref().map(DocumentRef).map_err(Error::Anyhow)
    }

    fn hex_cbor(&self) -> Result<String> {
        let bytes: Vec<u8> = self.0.to_bytes().map_err(Error::Anyhow)?;
        Ok(hex::encode(bytes))
    }
}

#[derive(uniffi::Object)]
struct DocumentRef(catalyst_signed_doc_lib::DocumentRef);

#[uniffi::export]
impl DocumentRef {
    fn id(&self) -> String {
        self.0.id().to_string()
    }

    fn ver(&self) -> String {
        self.0.ver().to_string()
    }
}
