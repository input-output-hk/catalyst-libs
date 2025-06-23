//! `reply` rule type impl.

use super::doc_ref::referenced_doc_check;
use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_doc_refs,
    CatalystSignedDocument, DocType,
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
        let context: &str = "Reply rule check";
        if let Self::Specified {
            exp_reply_type,
            optional,
        } = self
        {
            if let Some(reply_ref) = doc.doc_meta().reply() {
                let reply_validator = |ref_doc: CatalystSignedDocument| {
                    // Validate type
                    if !referenced_doc_check(&ref_doc, exp_reply_type, "reply", doc.report()) {
                        return false;
                    }

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
                    ref_doc_dr == doc_dr
                };
                return validate_doc_refs(reply_ref, provider, doc.report(), reply_validator).await;
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

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, DocumentRef,
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
        let missing_ref_replied_doc_id = UuidV7::new();
        let missing_ref_replied_doc_ver = UuidV7::new();
        let missing_type_replied_doc_ver = UuidV7::new();
        let missing_type_replied_doc_id = UuidV7::new();

        // Prepare provider documents
        {
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(valid_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(valid_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_reply_type.into()))
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        common_ref_id,
                        common_ref_ver,
                        Default::default(),
                    )]
                    .into(),
                ))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Reply doc with other `type` field
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(another_type_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(another_type_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        common_ref_id,
                        common_ref_ver,
                        Default::default(),
                    )]
                    .into(),
                ))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Missing `type` field in the referenced document
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_type_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_type_replied_doc_ver))
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        common_ref_id,
                        common_ref_ver,
                        Default::default(),
                    )]
                    .into(),
                ))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Missing `ref` field in the referenced document
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_ref_replied_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_ref_replied_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_reply_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();
        }

        // Create a document where `reply` field is required and referencing a valid document in
        // provider.
        let rule = ReplyRule::Specified {
            exp_reply_type: exp_reply_type.into(),
            optional: false,
        };

        // common_ref_id ref reply to valid_replied_doc_id. common_ref_id ref filed should match
        // valid_replied_doc_id ref field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    valid_replied_doc_id,
                    valid_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(
            rule.check(&doc, &provider).await.unwrap(),
            "{:?}",
            doc.problem_report()
        );

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
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    valid_replied_doc_id,
                    valid_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    another_type_replied_doc_id,
                    another_type_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    missing_ref_replied_doc_id,
                    missing_ref_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    missing_type_replied_doc_id,
                    missing_type_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // `ref` field does not align with the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    valid_replied_doc_id,
                    valid_replied_doc_ver,
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    Default::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    Default::default(),
                )]
                .into(),
            ))
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    common_ref_id,
                    common_ref_ver,
                    Default::default(),
                )]
                .into(),
            ))
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
            .with_metadata_field(SupportedField::Reply(
                vec![DocumentRef::new(ref_id, ref_ver, Default::default())].into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
