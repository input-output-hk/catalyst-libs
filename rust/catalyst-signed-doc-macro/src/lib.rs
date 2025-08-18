//! Catalyst Signed Documents code generation macro's from the defined `signed_doc.json`
//! spec.

mod error;
mod signed_doc_spec;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{error::process_error, signed_doc_spec::CatatalystSignedDocSpec};

/// Defines consts for all Catalyst Signed Documents types values
/// e.g.
/// ```ignore
/// pub const PROPOSAL: DocType = DocType::try_from_uuid(catalyst_types::uuid::uuid!(
///     "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
/// ));
/// ```
#[proc_macro]
pub fn catalyst_signed_documents_types_consts(
    _: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    catalyst_signed_documents_types_consts_impl()
        .unwrap_or_else(process_error)
        .into()
}

/// `catalyst_signed_documents_types_consts` macro implementation
fn catalyst_signed_documents_types_consts_impl() -> anyhow::Result<TokenStream> {
    let spec = CatatalystSignedDocSpec::load_signed_doc_spec()?;

    let mut consts_definitions = Vec::new();
    for (doc_name, doc_spec) in spec.docs {
        let const_type_name = doc_name
            .split_whitespace()
            .map(|word| word.to_uppercase())
            .collect::<Vec<_>>()
            .join("_");
        let const_type_name_ident = format_ident!("{const_type_name}",);
        let type_uuid = doc_spec.doc_type;

        let const_definition = quote! {
            /// Catalyst Signed Document type constant definition.
            pub const #const_type_name_ident: crate::DocType = match crate::DocType::try_from_uuid(catalyst_types::uuid::uuid!(#type_uuid)) {
                Ok(v) => v,
                Err(_) => panic!("invalid uuid v4 value"),
            };
        };
        consts_definitions.push(const_definition);
    }

    Ok(quote! {
        #(#consts_definitions)*
    })
}
