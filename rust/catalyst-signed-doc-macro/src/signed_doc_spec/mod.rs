//! Catalyst Signed Document spec type

// cspell: words pascalcase

pub(crate) mod content_type;
pub(crate) mod doc_ref;

use std::{collections::HashMap, ops::Deref};

use inflector::cases::pascalcase::to_pascal_case;
use proc_macro2::Ident;
use quote::format_ident;

/// Catalyst Signed Document spec representation struct
#[derive(serde::Deserialize)]
pub(crate) struct CatalystSignedDocSpec {
    /// A collection of document's supported content types
    #[serde(rename = "contentTypes")]
    #[allow(dead_code)]
    pub(crate) content_types: HashMap<ContentTypeTemplate, ContentTypeSpec>,
    /// A collection of document's specs
    pub(crate) docs: HashMap<DocumentName, DocSpec>,
}

// A thin wrapper over the RFC2046 content type strings.
#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct ContentTypeTemplate(pub(crate) String);

impl ContentTypeTemplate {
    /// returns a content type template as a `Ident` in the following form
    ///
    /// text/css; charset=utf-8; template=handlebars
    /// => `CssHandlebars`
    ///
    /// text/css; charset=utf-8
    /// => `Css`
    pub(crate) fn ident(&self) -> Ident {
        let raw = self.0.as_str();

        // split into parts like "text/css; charset=utf-8; template=handlebars"
        let mut parts = raw.split(';').map(str::trim);

        // first part is "type/subtype"
        let first = parts.next().unwrap_or_default(); // e.g. "text/css"
        let subtype = first.split('/').nth(1).unwrap_or_default(); // "css"

        // look for "template=..."
        let template = parts
            .find_map(|p| p.strip_prefix("template="))
            .map(to_pascal_case);

        // build PascalCase
        let mut ident = String::new();
        ident.push_str(&to_pascal_case(subtype));
        if let Some(t) = template {
            ident.push_str(&t);
        }

        format_ident!("{}", ident)
    }
}

/// Catalyst Signed Document supported content type declaration struct
#[derive(serde::Deserialize)]
pub(crate) struct ContentTypeSpec {
    /// CoAP Content-Formats
    #[allow(dead_code)]
    coap_type: Option<u32>,
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
    /// `headers` field
    pub(crate) headers: Headers,
    /// Document type metadata definitions
    pub(crate) metadata: Metadata,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Metadata {
    #[serde(rename = "ref")]
    pub(crate) doc_ref: doc_ref::Ref,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Headers {
    #[serde(rename = "content type")]
    pub(crate) content_type: content_type::ContentType,
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
