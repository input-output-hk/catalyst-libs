//! `RefRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, IsRequired};

/// Generating `RefRule` instantiation
pub(crate) fn rule(
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
        "application/cbor" => quote! { ContentTypeRule::Cbor },
        "application/cddl" => quote! { ContentTypeRule::Cddl },
        "application/json" => quote! { ContentTypeRule::Json },
        "application/json+schema" => quote! { ContentTypeRule::JsonSchema },
        "text/css; charset=utf-8" => quote! { ContentTypeRule::Css },
        "text/css; charset=utf-8; template=handlebars" => {
            quote! { ContentTypeRule::CssHandlebars }
        },
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
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported Content Type: {}",
                content_type.value
            ))
        },
    };

    Ok(quote! {
        crate::validator::rules::ContentTypeRule::Specified {
            exp: crate::validator::rules::#exp,
        }
    })
}
