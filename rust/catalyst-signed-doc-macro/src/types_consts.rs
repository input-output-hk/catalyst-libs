//! `catalyst_signed_documents_types_consts!` macro implementation

use catalyst_signed_doc_spec::CatalystSignedDocSpec;
use proc_macro2::TokenStream;
use quote::quote;

/// `catalyst_signed_documents_types_consts` macro implementation
pub(crate) fn catalyst_signed_documents_types_consts_impl() -> anyhow::Result<TokenStream> {
    let spec = CatalystSignedDocSpec::load_signed_doc_spec()?;

    let mut consts_definitions = Vec::new();
    for (doc_name, doc_spec) in spec.docs {
        if doc_spec.draft {
            continue;
        }
        let const_type_name_ident = doc_name.ident();
        let doc_name = doc_name.name();
        let type_uuid = doc_spec.doc_type;

        let const_definition = quote! {
            #[doc = #doc_name ]
            #[doc = "type constant definition"]
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
