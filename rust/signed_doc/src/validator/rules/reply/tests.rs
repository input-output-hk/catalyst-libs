use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    DocumentRefs, builder::tests::Builder, metadata::SupportedField,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_doc_ref,
};

#[test_case(
    |exp_type, provider| {
        let common_ref: DocumentRefs = vec![create_dummy_doc_ref()].into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Ref(common_ref.clone()))
            .with_metadata_field(SupportedField::Type(exp_type))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(common_ref))
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => true
    ;
    "valid reply to the correct document"
)]
#[test_case(
    |_, provider| {
        let common_ref: DocumentRefs = vec![create_dummy_doc_ref()].into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Ref(common_ref.clone()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(common_ref))
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reply to the document, with invalid `type` field"
)]
#[test_case(
    |_, provider| {
        let common_ref: DocumentRefs = vec![create_dummy_doc_ref()].into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Ref(common_ref.clone()))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(common_ref))
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
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
                vec![create_dummy_doc_ref()].into(),
            ))
            .with_metadata_field(SupportedField::Type(exp_type))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![create_dummy_doc_ref()].into(),
            ))
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reply to the document, with different `ref` field"
)]
#[test_case(
    |exp_type, provider| {
        let common_ref: DocumentRefs = vec![create_dummy_doc_ref()].into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_type))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(common_ref))
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reply to the document, with missing `ref` field"
)]
#[test_case(
    |_, provider| {
        let common_ref: DocumentRefs = vec![create_dummy_doc_ref()].into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Ref(common_ref.clone()))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![ref_doc.doc_ref().unwrap()].into(),
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
                vec![create_dummy_doc_ref()].into(),
            ))
            .with_metadata_field(SupportedField::Reply(
                vec![create_dummy_doc_ref()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reply to the missing document"
)]
#[tokio::test]
fn reply_specified_test(
    doc_gen: impl FnOnce(DocType, &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let exp_type: DocType = UuidV4::new().into();

    let doc = doc_gen(exp_type.clone(), &mut provider);

    let non_optional_res = ReplyRule::Specified {
        allowed_type: exp_type.clone(),
        optional: false,
    }
    .check_inner(&doc, &provider)
    .await
    .unwrap();

    let optional_res = ReplyRule::Specified {
        allowed_type: exp_type.clone(),
        optional: true,
    }
    .check_inner(&doc, &provider)
    .await
    .unwrap();

    assert_eq!(non_optional_res, optional_res);
    non_optional_res
}

#[tokio::test]
fn reply_specified_optional_test() {
    let provider = TestCatalystProvider::default();
    let rule = ReplyRule::Specified {
        allowed_type: UuidV4::new().into(),
        optional: true,
    };

    let doc = Builder::new().build();
    assert!(rule.check_inner(&doc, &provider).await.unwrap());

    let provider = TestCatalystProvider::default();
    let rule = ReplyRule::Specified {
        allowed_type: UuidV4::new().into(),
        optional: false,
    };

    let doc = Builder::new().build();
    assert!(!rule.check_inner(&doc, &provider).await.unwrap());
}

#[tokio::test]
fn reply_rule_not_specified_test() {
    let rule = ReplyRule::NotSpecified;
    let provider = TestCatalystProvider::default();

    let doc = Builder::new().build();
    assert!(rule.check_inner(&doc, &provider).await.unwrap());

    let doc = Builder::new()
        .with_metadata_field(SupportedField::Reply(vec![create_dummy_doc_ref()].into()))
        .build();
    assert!(!rule.check_inner(&doc, &provider).await.unwrap());
}
