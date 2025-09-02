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
pub(crate) struct Headers {
    #[serde(rename = "content type")]
    pub(crate) content_type: ContentType,    
}

/// `signed_doc.json` "content type" field JSON object
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
            "application/cbor" => quote! { ContentTypeRule::Cbor },
            "application/cddl" => quote! { ContentTypeRule::Cddl },
            "application/json" => quote! { ContentTypeRule::Json },
            "application/json+schema" => quote! { ContentTypeRule::JsonSchema },
            "text/css; charset=utf-8" => quote! { ContentTypeRule::Css },
            "text/css; charset=utf-8; template=handlebars" => quote! { ContentTypeRule::CssHandlebars },
            "text/html; charset=utf-8" => quote! { ContentTypeRule::Html },
            "text/html; charset=utf-8; template=handlebars" => {
                quote! { ContentTypeRule::HtmlHandlebars }
            },
            "text/markdown; charset=utf-8" => quote! { ContentTypeRule::Markdown },
            "text/markdown; charset=utf-8; template=handlebars" => {
                quote! { ContentTypeRule::MarkdownHandlebars }
            },
            "text/plain; charset=utf-8" => quote! { ContentTypeRule::Plain },
            "text/plain; charset=utf-8; template=handlebars" => {
                quote! { ContentTypeRule::PlainHandlebars }
            },
            _ => return Err(anyhow::anyhow!("Unsupported Content Type: {}", self.value)),
        };

        Ok(quote! {
            crate::validator::rules::ContentTypeRule {
                exp: crate::validator::rules::#exp,
            }
        })
    }
}
