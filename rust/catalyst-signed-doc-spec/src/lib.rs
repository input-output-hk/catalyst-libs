//! Catalyst Signed Document spec type

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

pub mod content_type;
pub mod doc_ref;

use std::{collections::HashMap, ops::Deref};

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub struct CatalystSignedDocSpec {
    /// A collection of document's supported content types
    #[serde(rename = "contentTypes")]
    #[allow(dead_code)]
    pub content_types: HashMap<String, ContentTypeSpec>,
    /// A collection of document's specs
    pub docs: HashMap<DocumentName, DocSpec>,
}

/// Catalyst Signed Document supported content type declaration struct
#[derive(serde::Deserialize)]
pub struct ContentTypeSpec {
    /// CoAP Content-Formats
    #[allow(dead_code)]
    coap_type: Option<u32>,
}

// A thin wrapper over the string document name values
#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
pub struct DocumentName(String);

impl DocumentName {
    /// returns document name
    pub fn name(&self) -> &str {
        &self.0
    }

    /// returns a document name as a `Ident` in the following form
    /// `"PROPOSAL_FORM_TEMPLATE"`
    pub fn ident(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            self.0
                .split_whitespace()
                .map(str::to_uppercase)
                .collect::<Vec<_>>()
                .join("_")
        )
    }
}

/// Specific document type definition
#[derive(serde::Deserialize)]
pub struct DocSpec {
    /// Document type UUID v4 value
    #[serde(rename = "type")]
    pub doc_type: String,
    /// `headers` field
    pub headers: Headers,
    /// Document type metadata definitions
    pub metadata: Metadata,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Metadata {
    #[serde(rename = "ref")]
    pub doc_ref: doc_ref::Ref,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct Headers {
    #[serde(rename = "content type")]
    pub content_type: content_type::ContentType,
}

/// "required" field definition
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::missing_docs_in_private_items)]
pub enum IsRequired {
    Yes,
    Excluded,
    Optional,
}

/// A helper type for deserialization "type" metadata field
pub struct DocTypes(Vec<DocumentName>);

impl Deref for DocTypes {
    type Target = Vec<DocumentName>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for DocTypes {
    #[allow(clippy::missing_docs_in_private_items)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum SingleOrVec {
            Single(DocumentName),
            Multiple(Vec<DocumentName>),
        }
        let value = Option::<SingleOrVec>::deserialize(deserializer)?;
        let result = match value {
            Some(SingleOrVec::Single(item)) => vec![item],
            Some(SingleOrVec::Multiple(items)) => items,
            None => vec![],
        };
        Ok(Self(result))
    }
}

impl CatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    pub fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let signed_doc_str = include_str!("../../../specs/signed_doc.json");
        let signed_doc_spec = serde_json::from_str(signed_doc_str)
            .map_err(|e| anyhow::anyhow!("Invalid Catalyst Signed Documents JSON Spec: {e}"))?;
        Ok(signed_doc_spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_signed_doc_spec_test() {
        assert!(CatalystSignedDocSpec::load_signed_doc_spec().is_ok());
    }
}
