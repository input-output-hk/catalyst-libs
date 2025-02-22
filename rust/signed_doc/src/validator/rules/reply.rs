//! `reply` rule type impl.

use catalyst_types::uuid::Uuid;

use crate::{
    CatalystSignedDocument, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc,
};

/// `reply` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ReplyRule {
    /// Is 'reply' specified
    Specified {
        /// expected `type` field of the replied doc
        exp_reply_type: Uuid,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'reply' is not specified
    NotSpecified,
}

impl ReplyRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_reply_type,
            optional,
        } = self
        {
            if let Some(reply) = doc.doc_meta().reply() {
                let reply_validator = |replied_doc: CatalystSignedDocument| {
                    if &replied_doc.doc_type()?.uuid() != exp_reply_type {
                        doc.report().invalid_value(
                            "reply",
                            replied_doc.doc_type()?.to_string().as_str(),
                            exp_reply_type.to_string().as_str(),
                            "Invalid referenced comment document type",
                        );
                        return Ok(false);
                    }
                    let Some(replied_doc_ref) = replied_doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Invalid referenced comment document");
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
                return validate_provided_doc(&reply, provider, doc.report(), reply_validator)
                    .await;
            } else if !optional {
                doc.report()
                    .missing_field("reply", "Document must have a reply field");
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(reply) = doc.doc_meta().reply() {
                doc.report().unknown_field(
                    "reply",
                    &reply.to_string(),
                    "Document does not expect to have a reply field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
