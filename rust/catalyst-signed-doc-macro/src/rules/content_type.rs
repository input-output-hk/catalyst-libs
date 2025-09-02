//! `RefRule` generation
use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, field::IsRequired};

/// Generating `RefRule` instantiation
pub(crate) fn ref_rule(ref_spec: &signed_doc_spec::field::ContentType) -> anyhow::Result<TokenStream> {
    let optional = match ref_spec.required {
        IsRequired::Yes => true,
        IsRequired::Optional => false,
        IsRequired::Excluded => {
            return Ok(quote! {
                crate::validator::rules::RefRule::NotSpecified
            });
        },
    };

    anyhow::ensure!(!ref_spec.doc_type.is_empty(), "'type' field should exists and has at least one entry for the required 'ref' metadata definition");

    let const_type_name_idents = ref_spec.doc_type.iter().map(|doc_name| {
        let const_type_name_ident = doc_name.ident();
        quote! {
            crate::doc_types::#const_type_name_ident
        }
    });
    let multiple = ref_spec.multiple.ok_or(anyhow::anyhow!(
        "'multiple' field should exists for the required 'ref' metadata definition"
    ))?;
    Ok(quote! {
        crate::validator::rules::ContentTypeRule {
            exp: ContentType::Json,
        }
    })
}