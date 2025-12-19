//! `reply` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{
    DocSpecs, DocumentName, is_required::IsRequired, metadata::reply::Reply,
};

use crate::{
    CatalystSignedDocument, DocType,
    providers::Provider,
    validator::{CatalystSignedDocumentValidationRule, rules::doc_ref::doc_refs_check},
};

/// `reply` field validation rule
#[derive(Debug)]
pub(crate) enum ReplyRule {
    /// Is 'reply' specified
    Specified {
        /// allowed `type` field of the replied doc
        allowed_type: DocType,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'reply' is not specified
    NotSpecified,
}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for ReplyRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider).await
    }
}

impl ReplyRule {
    /// Generating `ReplyRule` from specs
    pub(crate) fn new(
        docs: &DocSpecs,
        spec: &Reply,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                anyhow::ensure!(
                    spec.doc_type.is_empty() && !spec.multiple,
                    "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'reply'  metadata definition"
                );
                return Ok(Self::NotSpecified);
            },
        };

        anyhow::ensure!(
            !spec.multiple,
            "'multiple' field should be only set to false for the required 'reply' metadata definition"
        );

        let doc_name = &<&[DocumentName; 1]>::try_from(spec.doc_type.as_slice()).map_err(|_| anyhow::anyhow!("'type' field should exists and has only one entry for the required 'reply' metadata definition"))?[0];
        let docs_spec = docs.get(doc_name).ok_or(anyhow::anyhow!(
            "cannot find a document definition {doc_name}"
        ))?;
        let allowed_type = docs_spec.doc_type.as_str().parse()?;

        Ok(Self::Specified {
            allowed_type,
            optional,
        })
    }

    /// Field validation rule
    async fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let context: &str = "Reply rule check";
        if let Self::Specified {
            allowed_type: exp_reply_type,
            optional,
        } = self
        {
            if let Some(reply_ref) = doc.doc_meta().reply() {
                let reply_validator = |ref_doc: &CatalystSignedDocument| {
                    // Get `ref` from both the doc and the ref doc
                    let Some(ref_doc_dr) = ref_doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("Referenced doc `ref` field", context);
                        return false;
                    };

                    let Some(doc_dr) = doc.doc_meta().doc_ref() else {
                        doc.report().missing_field("Document `ref` field", context);
                        return false;
                    };

                    // Checking the ref field of ref doc, it should match the ref field of the doc
                    // If not record the error
                    if ref_doc_dr != doc_dr {
                        doc.report().invalid_value(
                            "ref",
                            &format!("Reference doc ref: {ref_doc_dr}"),
                            &format!("Doc ref: {doc_dr}"),
                            &format!("{context}, ref must be the same"),
                        );
                        return false;
                    }
                    true
                };

                return doc_refs_check(
                    reply_ref,
                    std::slice::from_ref(exp_reply_type),
                    false,
                    "reply",
                    provider,
                    doc.report(),
                    reply_validator,
                )
                .await;
            } else if !optional {
                doc.report().missing_field(
                    "reply",
                    &format!("{context}, document must have reply field"),
                );
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self
            && let Some(reply) = doc.doc_meta().reply()
        {
            doc.report().unknown_field(
                "reply",
                &reply.to_string(),
                &format!("{context}, document does not expect to have a reply field"),
            );
            return Ok(false);
        }

        Ok(true)
    }
}
