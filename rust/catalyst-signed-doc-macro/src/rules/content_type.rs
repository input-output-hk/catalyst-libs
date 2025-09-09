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
    let is_field_empty = field.value.is_none()
        || field
            .value
            .as_ref()
            .is_some_and(std::string::String::is_empty);

    if matches!(field.required, IsRequired::Excluded) {
        anyhow::ensure!(
            is_field_empty,
            "'value' field must not exist when 'required' is 'excluded'"
        );

        return Ok(quote! {
            crate::validator::rules::ContentTypeRule::NotSpecified
        });
    }

    if matches!(field.required, IsRequired::Yes) {
        anyhow::ensure!(!is_field_empty, "'value' field must exist");
    }

    let Some(value) = &field.value else {
        anyhow::bail!("'value' field must exist");
    };

    let template = ContentTypeTemplate(value.clone());
    let Some(_) = content_types.get(&template) else {
        return Err(anyhow::anyhow!("Unsupported Content Type: {}", value));
    };
    let ident = template.ident();

    Ok(quote! {
        crate::validator::rules::ContentTypeRule::Specified {
            exp: ContentType::#ident,
        }
    })
}
