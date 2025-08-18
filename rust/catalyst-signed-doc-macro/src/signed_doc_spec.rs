//! Catalyst Signed Document spec type

use std::collections::HashMap;

use anyhow::Context;

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub(crate) struct CatalystSignedDocSpec {
    pub(crate) docs: HashMap<DocumentName, DocSpec>,
}

/// A thin wrapper over the string document name values, mapping each of them from
/// "Proposal Form template" to "PROPOSAL_FORM_TEMPLATE"
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DocumentName(pub(crate) String);

impl<'de> serde::Deserialize<'de> for DocumentName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Ok(Self(
            String::deserialize(deserializer)?
                .split_whitespace()
                .map(|word| word.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
        ))
    }
}

/// Specific document type definition
#[derive(serde::Deserialize)]
pub(crate) struct DocSpec {
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
