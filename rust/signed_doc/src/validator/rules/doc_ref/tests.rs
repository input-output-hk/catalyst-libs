use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    DocLocator, DocumentRef,
};

#[test_case(
    |exp_types, provider| {
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
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
    "valid reference to the one correct document"
)]
#[test_case(
    |exp_types, provider| {
        let ref_doc_1 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(None, &ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
        .build();
        provider.add_document(None, &ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    ref_doc_1.doc_id().unwrap(),
                    ref_doc_1.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_2.doc_id().unwrap(),
                    ref_doc_2.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_3.doc_id().unwrap(),
                    ref_doc_3.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build()
    }
    => true
    ;
    "valid reference to the multiple documents"
)]
#[test_case(
    |exp_types, provider| {
        let ref_doc_1 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(None, &ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
        .build();
        provider.add_document(None, &ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    ref_doc_1.doc_id().unwrap(),
                    ref_doc_1.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_2.doc_id().unwrap(),
                    ref_doc_2.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_3.doc_id().unwrap(),
                    ref_doc_3.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the multiple documents, one of them invalid `type` field"
)]
#[test_case(
    |exp_types, provider| {
        let ref_doc_1 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(None, &ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .build();
        provider.add_document(None, &ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    ref_doc_1.doc_id().unwrap(),
                    ref_doc_1.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_2.doc_id().unwrap(),
                    ref_doc_2.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_3.doc_id().unwrap(),
                    ref_doc_3.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the multiple documents, one of them missing `type` field"
)]
#[test_case(
    |exp_types, provider| {
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(Some(DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default())), &ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
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
    "invalid reference to the document, which has different id and ver fields as stated in the `ref` field"
)]
#[test_case(
    |_, _| {
        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                ),
                ]
                .into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the missing one document"
)]
#[tokio::test]
async fn ref_multiple_specified_test(
    doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let exp_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

    let doc = doc_gen(&exp_types, &mut provider);

    let non_optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: true,
        optional: false,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    let optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: true,
        optional: true,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    assert_eq!(non_optional_res, optional_res);
    non_optional_res
}

#[test_case(
    |exp_types, provider| {
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
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
    "valid document with a single reference"
)]
#[test_case(
    |exp_types, provider| {
        let ref_doc_1 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(None, &ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(None, &ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
        .build();
        provider.add_document(None, &ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    ref_doc_1.doc_id().unwrap(),
                    ref_doc_1.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_2.doc_id().unwrap(),
                    ref_doc_2.doc_ver().unwrap(),
                    DocLocator::default(),
                ),
                DocumentRef::new(
                    ref_doc_3.doc_id().unwrap(),
                    ref_doc_3.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build()
    }
    => false
    ;
    "valid document with multiple references"
)]
#[tokio::test]
async fn ref_non_multiple_specified_test(
    doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let exp_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

    let doc = doc_gen(&exp_types, &mut provider);

    let non_optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: false,
        optional: false,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    let optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: false,
        optional: true,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    assert_eq!(non_optional_res, optional_res);
    non_optional_res
}

#[tokio::test]
async fn ref_specified_optional_test() {
    let provider = TestCatalystProvider::default();
    let rule = RefRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        multiple: true,
        optional: true,
    };

    let doc = Builder::new().build();
    assert!(rule.check(&doc, &provider).await.unwrap());

    let provider = TestCatalystProvider::default();
    let rule = RefRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        multiple: true,
        optional: false,
    };

    let doc = Builder::new().build();
    assert!(!rule.check(&doc, &provider).await.unwrap());
}

#[tokio::test]
async fn ref_rule_not_specified_test() {
    let rule = RefRule::NotSpecified;
    let provider = TestCatalystProvider::default();

    let doc = Builder::new().build();
    assert!(rule.check(&doc, &provider).await.unwrap());

    let ref_id = UuidV7::new();
    let ref_ver = UuidV7::new();
    let doc = Builder::new()
        .with_metadata_field(SupportedField::Ref(
            vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
        ))
        .build();
    assert!(!rule.check(&doc, &provider).await.unwrap());
}
