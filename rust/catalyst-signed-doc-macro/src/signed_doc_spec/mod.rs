//! Catalyst Signed Document spec type

pub(crate) mod doc_ref;
pub(crate) mod payload;
pub(crate) mod template;

use std::{collections::HashMap, ops::Deref};

use proc_macro2::Ident;
use quote::format_ident;

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub(crate) struct CatalystSignedDocSpec {
    /// A collection of document's specs
    pub(crate) docs: HashMap<DocumentName, DocSpec>,
}

// A thin wrapper over the string document name values
#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct DocumentName(String);

impl DocumentName {
    /// returns document name
    pub(crate) fn name(&self) -> &str {
        &self.0
    }

    /// returns a document name as a `Ident` in the following form
    /// `"PROPOSAL_FORM_TEMPLATE"`
    pub(crate) fn ident(&self) -> Ident {
        format_ident!(
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
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct DocSpec {
    #[serde(rename = "type")]
    pub(crate) doc_type: String,
    pub(crate) metadata: Metadata,
    pub(crate) payload: payload::Payload,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Metadata {
    #[serde(rename = "ref")]
    pub(crate) doc_ref: doc_ref::Ref,
    pub(crate) template: template::Template,
}

/// "required" field definition
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum IsRequired {
    Yes,
    Excluded,
    Optional,
}

/// A helper type for deserialization "type" metadata field
pub(crate) struct DocTypes(Vec<DocumentName>);

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
    // #[allow(dependency_on_unit_never_type_fallback)]
    pub(crate) fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let signed_doc_str = include_str!("../../../../specs/signed_doc.json");
        let signed_doc_spec = serde_json::from_str(signed_doc_str)?;
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
