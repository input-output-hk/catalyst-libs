//! Error definition

use proc_macro2::TokenStream;
use quote::quote;

/// Processes the provided `anyhow::Error` and parses it into the `TokenStream`
pub(crate) fn process_error(err: anyhow::Error) -> TokenStream {
    let err_str = err.to_string();
    if let Ok(err) = err.downcast::<syn::Error>() {
        err.into_compile_error()
    } else {
        quote!(#err_str)
    }
}
