//! `ref` rule type impl.

use catalyst_types::uuid::Uuid;

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `ref` field validation rule
pub(crate) struct RefRule {
    /// expected `type` field of the referenced doc
    pub(crate) ref_type: Uuid,
    /// optional flag for the `ref` field
    pub(crate) optional: bool,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let Some(doc_ref) = doc.doc_meta().doc_ref() {
            let ref_validator = |proposal_doc: CatalystSignedDocument| {
                if proposal_doc.doc_type()?.uuid() != self.ref_type {
                    doc.report().invalid_value(
                        "ref",
                        proposal_doc.doc_type()?.to_string().as_str(),
                        self.ref_type.to_string().as_str(),
                        "Invalid referenced proposal document type",
                    );
                    return Ok(false);
                }
                Ok(true)
            };
            return validate_provided_doc(&doc_ref, provider, doc.report(), ref_validator).await;
        } else if !self.optional {
            doc.report()
                .missing_field("ref", "Document must have a ref field");
            return Ok(false);
        }
        Ok(true)
    }
}
