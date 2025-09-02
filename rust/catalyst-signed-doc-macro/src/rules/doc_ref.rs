//! `RefRule` generation

use proc_macro2::TokenStream;
use quote::quote;

/// `signed_doc.json` "ref" field JSON defintion
#[derive(serde::Deserialize)]
pub(crate) struct Ref {}

/// Generating `RefRule` instantiation
pub(crate) fn ref_rule(_ref_spec: &Ref) -> anyhow::Result<TokenStream> {
    let res = quote! {
        crate::validator::rules::RefRule::NotSpecified
    };
    Ok(res)
}
