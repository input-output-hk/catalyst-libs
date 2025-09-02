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
pub(crate) struct Metadata {
    #[serde(rename = "content type")]
    pub(crate) content_type: ContentType,
}

/// `signed_doc.json` "ref" field JSON object
#[derive(serde::Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct ContentType {
    #[allow(dead_code)]
    pub(crate) required: IsRequired,
    pub(crate) value: String,
}

impl TryInto<TokenStream> for ContentType {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TokenStream, Self::Error> {
        let exp = match self.value.as_str() {
            "application/cbor" => quote! { ContentType::Cbor },
            "application/cddl" => quote! { ContentType::Cddl },
            "application/json" => quote! { ContentType::Json },
            "application/json+schema" => quote! { ContentType::JsonSchema },
            "text/css; charset=utf-8" => quote! { ContentType::Css },
            "text/css; charset=utf-8; template=handlebars" => quote! { ContentType::CssHandlebars },
            "text/html; charset=utf-8" => quote! { ContentType::Html },
            "text/html; charset=utf-8; template=handlebars" => {
                quote! { ContentType::HtmlHandlebars }
            },
            "text/markdown; charset=utf-8" => quote! { ContentType::Markdown },
            "text/markdown; charset=utf-8; template=handlebars" => {
                quote! { ContentType::MarkdownHandlebars }
            },
            "text/plain; charset=utf-8" => quote! { ContentType::Plain },
            "text/plain; charset=utf-8; template=handlebars" => {
                quote! { ContentType::PlainHandlebars }
            },
            _ => return Err(anyhow::anyhow!("Unsupported Content Type: {}", self.value)),
        };

        Ok(quote! {
            crate::validator::rules::ContentTypeRule {
                exp: #exp,
            }
        })
    }
}
