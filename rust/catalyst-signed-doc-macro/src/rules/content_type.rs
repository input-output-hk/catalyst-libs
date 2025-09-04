//! `ContentTypeRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, IsRequired};

/// Generating `ContentTypeRule` instantiation
pub(crate) fn into_rule(
    content_type: &signed_doc_spec::content_type::ContentType
) -> anyhow::Result<TokenStream> {
    if matches!(content_type.required, IsRequired::Excluded) {
        anyhow::ensure!(
            content_type.value.is_empty(),
            "'value' field must not exist when 'required' is 'excluded'"
        );

        return Ok(quote! {
            crate::validator::rules::ContentTypeRule::Unspecified
        });
    }

    if matches!(content_type.required, IsRequired::Yes) {
        anyhow::ensure!(!content_type.value.is_empty(), "'value' field must exist");
    }

    let exp = match content_type.value.as_str() {
        "application/cbor" => quote! { ContentType::Cbor },
        "application/cddl" => quote! { ContentType::Cddl },
        "application/json" => quote! { ContentType::Json },
        "application/json+schema" => quote! { ContentType::JsonSchema },
        "text/css; charset=utf-8" => quote! { ContentType::Css },
        "text/css; charset=utf-8; template=handlebars" => {
            quote! { ContentType::CssHandlebars }
        },
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
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported Content Type: {}",
                content_type.value
            ))
        },
    };

    Ok(quote! {
        crate::validator::rules::ContentTypeRule::Specified {
            exp: #exp,
        }
    })
}
