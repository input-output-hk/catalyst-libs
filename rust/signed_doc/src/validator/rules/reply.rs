//! `reply` rule type impl.

use super::doc_ref::referenced_doc_check;
use crate::{
    metadata::DocType, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
};

/// `reply` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ReplyRule {
    /// Is 'reply' specified
    Specified {
        /// expected `type` field of the replied doc
        exp_reply_type: DocType,
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
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_reply_type,
            optional,
        } = self
        {
            if let Some(reply) = doc.doc_meta().reply() {
                let reply_validator = |replied_doc: CatalystSignedDocument| {
                    if !referenced_doc_check(&replied_doc, exp_reply_type, "reply", doc.report()) {
                        return false;
                    }
                    let Some(doc_ref) = doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Document must have a ref field");
                        return false;
                    };

                    let Some(replied_doc_ref) = replied_doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Referenced document must have ref field");
                        return false;
                    };

                    if replied_doc_ref.id != doc_ref.id {
                        doc.report().invalid_value(
                                "reply",
                                doc_ref.id .to_string().as_str(),
                                replied_doc_ref.id.to_string().as_str(),
                                "Invalid referenced document. Document ID should aligned with the replied document.",
                            );
                        return false;
                    }

                    true
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

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{
        metadata::SupportedField, providers::tests::TestCatalystSignedDocumentProvider, Builder,
        DocumentRef,
    };

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_reply_type = UuidV4::new();
        let common_ref_id = UuidV7::new();
        let common_ref_ver = UuidV7::new();

        let valid_replied_doc_id = UuidV7::new();
        let valid_replied_doc_ver = UuidV7::new();
        let another_type_replied_doc_ver = UuidV7::new();
        let another_type_replied_doc_id = UuidV7::new();
        let missing_ref_replied_doc_ver = UuidV7::new();
        let missing_ref_replied_doc_id = UuidV7::new();
        let missing_type_replied_doc_ver = UuidV7::new();
        let missing_type_replied_doc_id = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(valid_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(valid_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_reply_type.into()))
                .with_metadata_field(SupportedField::Ref(DocumentRef {
                    id: common_ref_id,
                    ver: common_ref_ver,
                }))
                .build();
            provider.add_document(ref_doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(another_type_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(another_type_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .with_metadata_field(SupportedField::Ref(DocumentRef {
                    id: common_ref_id,
                    ver: common_ref_ver,
                }))
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `ref` field in the referenced document
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_ref_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_ref_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_reply_type.into()))
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_type_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_type_replied_doc_ver))
                .with_metadata_field(SupportedField::Ref(DocumentRef {
                    id: common_ref_id,
                    ver: common_ref_ver,
                }))
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ReplyRule::Specified {
            exp_reply_type: exp_reply_type.into(),
            optional: false,
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: valid_replied_doc_id,
                ver: valid_replied_doc_ver,
            }))
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `reply` field is missing, but its optional
        let rule = ReplyRule::Specified {
            exp_reply_type: exp_reply_type.into(),
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `reply` field, but its required
        let rule = ReplyRule::Specified {
            exp_reply_type: exp_reply_type.into(),
            optional: false,
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: valid_replied_doc_id,
                ver: valid_replied_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: another_type_replied_doc_id,
                ver: another_type_replied_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: missing_ref_replied_doc_id,
                ver: missing_type_replied_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: missing_type_replied_doc_id,
                ver: missing_type_replied_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // `ref` field does not align with the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: UuidV7::new(),
                ver: UuidV7::new(),
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: valid_replied_doc_id,
                ver: valid_replied_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(DocumentRef {
                id: common_ref_id,
                ver: common_ref_ver,
            }))
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: UuidV7::new(),
                ver: UuidV7::new(),
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn reply_rule_not_specified_test() {
        let rule = ReplyRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(DocumentRef {
                id: ref_id,
                ver: ref_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
