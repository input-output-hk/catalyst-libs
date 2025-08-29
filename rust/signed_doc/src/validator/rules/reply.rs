//! `reply` rule type impl.

use crate::{
    providers::CatalystSignedDocumentProvider, validator::rules::doc_ref::doc_refs_check,
    CatalystSignedDocument, DocType,
};

/// `reply` field validation rule
#[derive(Debug)]
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
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let context: &str = "Reply rule check";
        if let Self::Specified {
            exp_reply_type,
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
        if let Self::NotSpecified = self {
            if let Some(reply) = doc.doc_meta().reply() {
                doc.report().unknown_field(
                    "reply",
                    &reply.to_string(),
                    &format!("{context}, document does not expect to have a reply field"),
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
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, DocLocator, DocumentRef,
        DocumentRefs,
    };

    #[test_case(
        |exp_type, provider| {
            let common_ref: DocumentRefs = vec![DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            )]
            .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Ref(common_ref.clone()))
                .with_metadata_field(SupportedField::Type(exp_type))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(common_ref))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => true
        ;
        "valid reply to the correct document"
    )]
    #[test_case(
        |_, provider| {
            let common_ref: DocumentRefs = vec![DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            )]
            .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Ref(common_ref.clone()))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(common_ref))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reply to the document, with invalid `type` field"
    )]
    #[test_case(
        |_, provider| {
            let common_ref: DocumentRefs = vec![DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            )]
            .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Ref(common_ref.clone()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(common_ref))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reply to the document, with missing `type` field"
    )]
    #[test_case(
        |exp_type, provider| {
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .with_metadata_field(SupportedField::Type(exp_type))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reply to the document, with different `ref` field"
    )]
    #[test_case(
        |exp_type, provider| {
            let common_ref: DocumentRefs = vec![DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            )]
            .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_type))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(common_ref))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reply to the document, with missing `ref` field"
    )]
    #[test_case(
        |_, provider| {
            let common_ref: DocumentRefs = vec![DocumentRef::new(
                UuidV7::new(),
                UuidV7::new(),
                DocLocator::default(),
            )]
            .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Ref(common_ref.clone()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "missing `ref` field and reply to the valid document"
    )]
    #[test_case(
        |_, _| {
            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reply to the missing document"
    )]
    #[tokio::test]
    async fn reply_specified_test(
        doc_gen: impl FnOnce(DocType, &mut TestCatalystSignedDocumentProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_type: DocType = UuidV4::new().into();

        let doc = doc_gen(exp_type.clone(), &mut provider);

        let non_optional_res = ReplyRule::Specified {
            exp_reply_type: exp_type.clone(),
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = ReplyRule::Specified {
            exp_reply_type: exp_type.clone(),
            optional: true,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        assert_eq!(non_optional_res, optional_res);
        non_optional_res
    }

    #[tokio::test]
    async fn reply_specified_optional_test() {
        let provider = TestCatalystSignedDocumentProvider::default();
        let rule = ReplyRule::Specified {
            exp_reply_type: UuidV4::new().into(),
            optional: true,
        };

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let provider = TestCatalystSignedDocumentProvider::default();
        let rule = ReplyRule::Specified {
            exp_reply_type: UuidV4::new().into(),
            optional: false,
        };

        let doc = Builder::new().build();
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
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
