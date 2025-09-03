//! `ContentRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self};

/// Generating `ContentRule` instantiation
pub(crate) fn content_rule(
    _payload_spec: &signed_doc_spec::payload::Payload,
    _template_spec: &signed_doc_spec::template::Template,
) -> anyhow::Result<TokenStream> {
    Ok(quote! {
        crate::validator::rules::ContentRule::NotSpecified
    })
}
