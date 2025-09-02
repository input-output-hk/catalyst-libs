//! Structs for JSON fields.

/// "required" field definition
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum IsRequired {
    Yes,
    Excluded,
    Optional,
}

/// Document's metadata fields definition
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct MetadataNode {
    #[serde(rename = "ref")]
    pub(crate) content_type: ContentType,
}

/// `signed_doc.json` "ref" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct ContentType {
    pub(crate) required: IsRequired,
    pub(crate) value: Vec<String>,
}
