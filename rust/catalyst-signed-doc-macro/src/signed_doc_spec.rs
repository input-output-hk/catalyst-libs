//! Catalyst Signed Document spec type

use std::collections::HashMap;

use anyhow::Context;

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub(crate) struct CatatalystSignedDocSpec {
    pub(crate) docs: HashMap<String, DocSpec>,
}

/// Specific document type definition
#[derive(serde::Deserialize)]
pub(crate) struct DocSpec {
    #[serde(rename = "type")]
    pub(crate) doc_type: String,
}

impl CatatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    // #[allow(dependency_on_unit_never_type_fallback)]
    pub(crate) fn load_signed_doc_spec() -> anyhow::Result<CatatalystSignedDocSpec> {
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
        assert!(CatatalystSignedDocSpec::load_signed_doc_spec().is_ok());
    }
}
