//! `TemplateRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::{self, IsRequired};

/// Generating `TemplateRule` instantiation
pub(crate) fn template_rule(
    spec: &signed_doc_spec::template::Template
) -> anyhow::Result<TokenStream> {
    if let IsRequired::Excluded = spec.required {
        anyhow::ensure!(
            spec.doc_type.is_none() && spec.multiple.is_none(),
            "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'template'  metadata definition"
        );
        return Ok(quote! {
            crate::validator::rules::TemplateRule::NotSpecified
        });
    }

    anyhow::ensure!(
        spec.multiple.is_some_and(|v| !v),
        "'multiple' must be `false` for 'template' metadata definition"
    );
    anyhow::ensure!(
        spec.required != IsRequired::Optional,
        "'required' field cannot been 'optional' for 'template'  metadata definition"
    );

    let const_type_name_ident = spec
        .doc_type
        .as_ref()
        .ok_or(anyhow::anyhow!(
            "'type' field should exists for the required 'template' metadata definition"
        ))?
        .ident();

    Ok(quote! {
        crate::validator::rules::TemplateRule::Specified {
            allowed_type: crate::doc_types::#const_type_name_ident,
        }
    })
}
