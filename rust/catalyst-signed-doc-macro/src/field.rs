//! Structs for JSON fields.
use proc_macro2::TokenStream;
use quote::quote;

/// "required" field definition
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum IsRequired {
    Yes,
    Excluded,
    Optional,
}

impl TryInto<TokenStream> for IsRequired {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TokenStream, Self::Error> {
        match self {
            Self::Yes => Ok(quote! { true }),
            Self::Optional => Ok(quote! { false }),
            Self::Excluded => {
                return Ok(quote! {
                    crate::validator::rules::RefRule::NotSpecified
                });
            },
        }
    }
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
