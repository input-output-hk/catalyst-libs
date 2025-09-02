//! `RefRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, IsRequired};

/// Generating `RefRule` instantiation
pub(crate) fn ref_rule(ref_spec: &signed_doc_spec::doc_ref::Ref) -> anyhow::Result<TokenStream> {
    match ref_spec.required {
        IsRequired::Yes => {
            let doc_name = ref_spec.doc_type.as_ref().ok_or(anyhow::anyhow!(
                "'type' field should exists for the required 'ref' metadata definition"
            ))?;
            let const_type_name_ident = doc_name.ident();
            Ok(quote! {
                crate::validator::rules::RefRule::Specified {
                    exp_ref_types: vec![ #const_type_name_ident ]
                    optional: false,
                }
            })
        },
        IsRequired::Optional => {
            let doc_name = ref_spec.doc_type.as_ref().ok_or(anyhow::anyhow!(
                "'type' field should exists for the required 'ref' metadata definition"
            ))?;
            let const_type_name_ident = doc_name.ident();
            Ok(quote! {
                crate::validator::rules::RefRule::Specified {
                    exp_ref_types: vec![ #const_type_name_ident ]
                    optional: true,
                }
            })
        },
        IsRequired::Excluded => {
            Ok(quote! {
                crate::validator::rules::RefRule::NotSpecified
            })
        },
    }
}
