//! Catalyst Signed Document spec type

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

pub mod copyright;
pub mod doc_types;
pub mod headers;
pub mod is_required;
pub mod metadata;
pub mod signers;

use std::{collections::HashMap, fmt::Display};

use build_info as build_info_lib;

use crate::{copyright::Copyright, headers::Headers, metadata::Metadata, signers::Signers};

build_info_lib::build_info!(pub(crate) fn build_info);

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub struct CatalystSignedDocSpec {
    pub docs: HashMap<DocumentName, DocSpec>,
    pub copyright: Copyright,
}

// A thin wrapper over the string document name values
#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
pub struct DocumentName(String);

impl Display for DocumentName {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl DocumentName {
    /// returns document name
    #[must_use]
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
    pub signers: Signers,
}

impl CatalystSignedDocSpec {
    /// Loading a Catalyst Signed Documents spec from the `signed_doc.json`
    ///
    /// # Errors
    ///  - `signed_doc.json` filed loading and JSON parsing errors
    ///  - `catalyst-signed-doc-spec` crate version doesn't  align with the latest version
    ///    of the `signed_doc.json`
    pub fn load_signed_doc_spec() -> anyhow::Result<CatalystSignedDocSpec> {
        let signed_doc_str = include_str!("../../../specs/signed_doc.json");
        let signed_doc_spec: CatalystSignedDocSpec = serde_json::from_str(signed_doc_str)
            .map_err(|e| anyhow::anyhow!("Invalid Catalyst Signed Documents JSON Spec: {e}"))?;

        let crate_version = build_info().crate_info.version.to_string();
        let latest_version = signed_doc_spec
            .copyright
            .versions
            .last()
            .ok_or(anyhow::anyhow!(
                "'versions' list must have at least one entry"
            ))?;
        anyhow::ensure!(latest_version.version == crate_version, "crate version should align with the latest version of the Catalyst Signed Documents specification");

        Ok(signed_doc_spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_signed_doc_spec_test() {
        CatalystSignedDocSpec::load_signed_doc_spec().unwrap();
    }
}
