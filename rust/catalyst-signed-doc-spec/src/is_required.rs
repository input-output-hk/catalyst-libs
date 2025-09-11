//! 'required' field allowed values definition

/// "required" field definition
#[derive(serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::missing_docs_in_private_items)]
pub enum IsRequired {
    Yes,
    Excluded,
    Optional,
}
