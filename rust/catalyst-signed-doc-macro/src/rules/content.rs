//! `ContentRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, IsRequired};

/// Generating `ContentRule` instantiation
pub(crate) fn content_rule(spec: &signed_doc_spec::DocSpec) -> anyhow::Result<TokenStream> {
    if let IsRequired::Excluded = spec.payload.required {
        anyhow::ensure!(
            spec.metadata.template.required == IsRequired::Excluded,
            "if document 'payload' is excluded, 'template' field must be excluded as well"
        );
        return Ok(quote! {
            crate::validator::rules::ContentRule::NotSpecified
        });
    }

    

    Ok(quote! {
        crate::validator::rules::ContentRule::NotSpecified
    })
}
