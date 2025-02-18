//! `reply` rule type impl.

use catalyst_types::uuid::Uuid;
use futures::{future::BoxFuture, FutureExt};

use crate::{
    providers::CatalystSignedDocumentProvider,
    validator::{utils::validate_provided_doc, ValidationRule},
    CatalystSignedDocument,
};

/// `reply` field validation rule
pub(crate) struct ReplyRule {
    /// expected `type` field of the replied doc
    pub(crate) reply_type: Uuid,
    /// optional flag for the `ref` field
    pub(crate) optional: bool,
}
impl<Provider> ValidationRule<Provider> for ReplyRule
where Provider: 'static + CatalystSignedDocumentProvider
{
    fn check<'a>(
        &'a self, doc: &'a CatalystSignedDocument, provider: &'a Provider,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        async {
            if let Some(reply) = doc.doc_meta().reply() {
                let reply_validator = |replied_doc: CatalystSignedDocument| {
                    if replied_doc.doc_type()?.uuid() != self.reply_type {
                        doc.report().invalid_value(
                            "reply",
                            replied_doc.doc_type()?.to_string().as_str(),
                            self.reply_type.to_string().as_str(),
                            "Invalid referenced comment document type",
                        );
                        return Ok(false);
                    }
                    let Some(replied_doc_ref) = replied_doc.doc_meta().doc_ref() else {
                        doc.report().missing_field("ref", "Invalid referenced comment document");
                        return Ok(false);
                    };

                    if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                        if replied_doc_ref.id != doc_ref.id {
                            doc.report().invalid_value(
                                "reply",
                                doc_ref.id .to_string().as_str(),
                                replied_doc_ref.id.to_string().as_str(),
                                "Invalid referenced comment document. Document ID should aligned with the replied comment.",
                            );
                            return Ok(false);
                        }
                    }
        
                    Ok(true)
                };
                return validate_provided_doc(&reply, provider, doc.report(), reply_validator).await;
            } else if !self.optional {
                doc.report().missing_field("reply", "Document must have a reply field");
                return Ok(false);
            }
            Ok(true)
        }
        .boxed()
    }
}
