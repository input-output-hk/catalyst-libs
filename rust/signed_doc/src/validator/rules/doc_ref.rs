//! `ref` rule type impl.

use catalyst_types::uuid::Uuid;

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `ref` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RefRule {
    /// Is 'ref' specified
    Specified {
        /// expected `type` field of the referenced doc
        exp_ref_type: Uuid,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_ref_type,
            optional,
        } = self
        {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |ref_doc: CatalystSignedDocument| {
                    if &ref_doc.doc_type()?.uuid() != exp_ref_type {
                        doc.report().invalid_value(
                            "ref",
                            ref_doc.doc_type()?.to_string().as_str(),
                            exp_ref_type.to_string().as_str(),
                            "Invalid referenced document type",
                        );
                        return Ok(false);
                    }
                    Ok(true)
                };
                return validate_provided_doc(&doc_ref, provider, doc.report(), ref_validator)
                    .await;
            } else if !optional {
                doc.report()
                    .missing_field("ref", "Document must have a ref field");
                return Ok(false);
            }
        }
        if &Self::NotSpecified == self {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                doc.report().unknown_field(
                    "ref",
                    &doc_ref.to_string(),
                    "Document does not expect to have a ref field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
