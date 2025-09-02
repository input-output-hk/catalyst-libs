//! Catalyst Signed Document spec type

use std::collections::HashMap;

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
pub(crate) struct DocSpec {
    /// Document type UUID v4 value
    #[serde(rename = "type")]
    pub(crate) doc_type: String,

    /// Document type metadata definitions
    pub(crate) metadata: Metadata,
}

/// Document's metadata fields defintion
#[derive(serde::Deserialize)]
pub(crate) struct Metadata {
    #[serde(rename = "ref")]
    pub(crate) doc_ref: crate::rules::doc_ref::Ref,
}

impl CatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    // #[allow(dependency_on_unit_never_type_fallback)]
    pub(crate) fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let signed_doc_str = include_str!("../../../specs/signed_doc.json");
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
