//! `ContentRule` generation

use proc_macro2::TokenStream;
use quote::quote;

use crate::signed_doc_spec;

/// Generating `ContentRule` instantiation
pub(crate) fn content_rule(
    spec: &signed_doc_spec::payload::Payload
) -> anyhow::Result<TokenStream> {
    if spec.nil {
        anyhow::ensure!(
            spec.schema.is_none(),
            "'schema' field could not been specified when 'nil' is 'true'"
        );
        return Ok(quote! {
            crate::validator::rules::ContentRule::Nil
        });
    }

    if let Some(schema) = &spec.schema {
        let schema_str = schema.to_string();
        Ok(quote! {
            crate::validator::rules::ContentRule::StaticSchema(
                crate::validator::rules::ContentSchema::Json(
                    json_schema::JsonSchema::try_from(
                        &serde_json::from_str(
                            #schema_str
                        ).expect("Must be a valid json")
                    ).expect("Must be a valid JSON scheme")
                )
            )
        })
    } else {
        Ok(quote! {
            crate::validator::rules::ContentRule::NotNil
        })
    }
}
