//! catalyst_signed_documents_rules! macro implementation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::CatalystSignedDocSpec;

/// `catalyst_signed_documents_rules` macro implementation
pub(crate) fn catalyst_signed_documents_rules_impl() -> anyhow::Result<TokenStream> {
    let _spec = CatalystSignedDocSpec::load_signed_doc_spec()?;

    Ok(quote! {
        /// Returns an iterator with all defined Catalyst Signed Documents validation rules per corresponding document type
        fn documents_rules() -> impl Iterator<Item = (DocType, Rules)> {
            [].into_iter()
        }
    })
}
