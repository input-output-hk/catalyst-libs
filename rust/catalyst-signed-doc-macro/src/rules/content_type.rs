//! `ContentTypeRule` generation

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, ContentTypeSpec, ContentTypeTemplate, IsRequired};

/// Generating `ContentTypeRule` instantiation
pub(crate) fn into_rule(
    content_types: &HashMap<ContentTypeTemplate, ContentTypeSpec>,
    field: &signed_doc_spec::content_type::ContentType,
) -> anyhow::Result<TokenStream> {
    if matches!(field.required, IsRequired::Excluded) {
        anyhow::ensure!(
            field.value.is_empty(),
            "'value' field must not exist when 'required' is 'excluded'"
        );

        return Ok(quote! {
            crate::validator::rules::ContentTypeRule::NotSpecified
        });
    }

    if matches!(field.required, IsRequired::Yes) {
        anyhow::ensure!(!field.value.is_empty(), "'value' field must exist");
    }

    let template = ContentTypeTemplate(field.value.clone());
    let Some(_) = content_types.get(&template) else {
        return Err(anyhow::anyhow!("Unsupported Content Type: {}", field.value));
    };
    let ident = template.ident();

    Ok(quote! {
        crate::validator::rules::ContentTypeRule::Specified {
            exp: ContentType::#ident,
        }
    })
}
