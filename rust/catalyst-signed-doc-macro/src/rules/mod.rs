//! `catalyst_signed_documents_rules!` macro implementation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec::CatalystSignedDocSpec;

/// `catalyst_signed_documents_rules` macro implementation
pub(crate) fn catalyst_signed_documents_rules_impl() -> anyhow::Result<TokenStream> {
    let spec = CatalystSignedDocSpec::load_signed_doc_spec()?;

    let mut rules_definitions = Vec::new();
    for (doc_name, _doc_spec) in spec.docs {
        let const_type_name_ident = doc_name.ident();

        // TODO: implement a proper initialization for all specific validation rules
        let rules = quote! {
            crate::validator::rules::Rules {
                id: crate::validator::rules::IdRule,
                ver: crate::validator::rules::VerRule,
                content_type: crate::validator::rules::ContentTypeRule {
                    exp: ContentType::Json,
                },
                content_encoding: crate::validator::rules::ContentEncodingRule {
                    exp: ContentEncoding::Brotli,
                    optional: false,
                },
                content: crate::validator::rules::ContentRule::NotSpecified,
                parameters: crate::validator::rules::ParametersRule::NotSpecified,
                doc_ref: crate::validator::rules::RefRule::NotSpecified,
                reply: crate::validator::rules::ReplyRule::NotSpecified,
                section: crate::validator::rules::SectionRule::NotSpecified,
                kid: crate::validator::rules::SignatureKidRule {
                    exp: &[],
                },
                signature: crate::validator::rules::SignatureRule {
                    mutlisig: false
                },
            }
        };

        let rule_definition = quote! {
            (crate::doc_types::#const_type_name_ident, #rules),
        };
        rules_definitions.push(rule_definition);
    }

    Ok(quote! {
        /// Returns an iterator with all defined Catalyst Signed Documents validation rules per corresponding document type
        fn documents_rules() -> impl Iterator<Item = (crate::DocType, crate::validator::rules::Rules)> {
            [ #(#rules_definitions)* ].into_iter()
        }
    })
}
