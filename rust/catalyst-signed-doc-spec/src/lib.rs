//! Catalyst Signed Document spec type

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

pub mod doc_types;
pub mod headers;
pub mod is_required;
pub mod metadata;

use std::collections::HashMap;

use build_info;

use crate::{headers::Headers, metadata::Metadata};

build_info::build_info!(pub(crate) fn build_info);

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub struct CatalystSignedDocSpec {
    /// A collection of document's specs
    pub docs: HashMap<DocumentName, DocSpec>,
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
    #[serde(rename = "type")]
    pub doc_type: String,
    pub headers: Headers,
    pub metadata: Metadata,
}

impl CatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    pub fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let crate_version = build_info().crate_info.version.to_string();

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
