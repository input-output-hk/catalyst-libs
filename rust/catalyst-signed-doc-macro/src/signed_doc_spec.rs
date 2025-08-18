//! Catalyst Signed Document spec type

use std::collections::HashMap;

use anyhow::Context;
use proc_macro2::Ident;
use quote::format_ident;

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub(crate) struct CatalystSignedDocSpec {
    /// A collection of document's specs
    pub(crate) docs: DocumentSpecs,
}

/// A collection of document's specs
#[derive(serde::Deserialize)]
pub(crate) struct DocumentSpecs(HashMap<String, DocSpec>);

impl IntoIterator for DocumentSpecs {
    type IntoIter = std::iter::Map<
        <HashMap<String, DocSpec> as IntoIterator>::IntoIter,
        fn((String, DocSpec)) -> (Ident, String, DocSpec),
    >;
    type Item = (Ident, String, DocSpec);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|(doc_name, spec)| {
            let doc_name_ident = doc_name
                .split_whitespace()
                .map(str::to_uppercase)
                .collect::<Vec<_>>()
                .join("_");
            let doc_name_ident = format_ident!("{}", doc_name_ident);
            (doc_name_ident, doc_name, spec)
        })
    }
}

/// Specific document type definition
#[derive(serde::Deserialize)]
pub(crate) struct DocSpec {
    /// Document type UUID v4 value
    #[serde(rename = "type")]
    pub(crate) doc_type: String,
}

impl CatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    // #[allow(dependency_on_unit_never_type_fallback)]
    pub(crate) fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let signed_doc_str = include_str!("../../../specs/signed_doc.json");
        let signed_doc_spec = serde_json::from_str(signed_doc_str)
            .context("Catalyst Signed Documents spec must be a JSON object")?;
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
