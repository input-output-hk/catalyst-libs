use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_doc_ref,
};

#[test_case(
    |exp_types, provider| {
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![ref_doc.doc_ref().unwrap()].into(),
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
        provider.add_document(&ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(&ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
        .build();
        provider.add_document(&ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![
                    ref_doc_1.doc_ref().unwrap(),
                    ref_doc_2.doc_ref().unwrap(),
                    ref_doc_3.doc_ref().unwrap(),
                ]
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
        provider.add_document(&ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(&ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
        .build();
        provider.add_document(&ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![
                    ref_doc_1.doc_ref().unwrap(),
                    ref_doc_2.doc_ref().unwrap(),
                    ref_doc_3.doc_ref().unwrap(),
                ]
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
        provider.add_document(&ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(&ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .build();
        provider.add_document(&ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![
                    ref_doc_1.doc_ref().unwrap(),
                    ref_doc_2.doc_ref().unwrap(),
                    ref_doc_3.doc_ref().unwrap(),
                ]
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

        let new_ref = create_dummy_doc_ref();
        provider.add_document_with_ref(new_ref.clone(), &ref_doc);

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![new_ref].into(),
            ))
            .build()
    }
    => false
    ;
    "invalid reference in the `ref` field to the document, which is different with the fetched document"
)]
#[test_case(
    |_, _| {
        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![create_dummy_doc_ref()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the missing one document"
)]
fn ref_multiple_specified_test(
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
    .check_inner(&doc, &provider)
    .unwrap();

    let optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: true,
        optional: true,
    }
    .check_inner(&doc, &provider)
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
        provider.add_document(&ref_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![ref_doc.doc_ref().unwrap()].into(),
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
        provider.add_document(&ref_doc_1).unwrap();
        let ref_doc_2 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
            .build();
        provider.add_document(&ref_doc_2).unwrap();
        let ref_doc_3 = Builder::new()
        .with_metadata_field(SupportedField::Id(UuidV7::new()))
        .with_metadata_field(SupportedField::Ver(UuidV7::new()))
        .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
        .build();
        provider.add_document(&ref_doc_3).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![
                    ref_doc_1.doc_ref().unwrap(),
                    ref_doc_2.doc_ref().unwrap(),
                    ref_doc_3.doc_ref().unwrap(),
                ]
                .into(),
            ))
            .build()
    }
    => false
    ;
    "valid document with multiple references"
)]
fn ref_non_multiple_specified_test(
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
    .check_inner(&doc, &provider)
    .unwrap();

    let optional_res = RefRule::Specified {
        allowed_type: exp_types.to_vec(),
        multiple: false,
        optional: true,
    }
    .check_inner(&doc, &provider)
    .unwrap();

    assert_eq!(non_optional_res, optional_res);
    non_optional_res
}

#[test]
fn ref_specified_optional_test() {
    let provider = TestCatalystProvider::default();
    let rule = RefRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        multiple: true,
        optional: true,
    };

    let doc = Builder::new().build();
    assert!(rule.check_inner(&doc, &provider).unwrap());

    let provider = TestCatalystProvider::default();
    let rule = RefRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        multiple: true,
        optional: false,
    };

    let doc = Builder::new().build();
    assert!(!rule.check_inner(&doc, &provider).unwrap());
}

#[test]
fn ref_rule_not_specified_test() {
    let rule = RefRule::NotSpecified;
    let provider = TestCatalystProvider::default();

    let doc = Builder::new().build();
    assert!(rule.check_inner(&doc, &provider).unwrap());

    let doc = Builder::new()
        .with_metadata_field(SupportedField::Ref(vec![create_dummy_doc_ref()].into()))
        .build();
    assert!(!rule.check_inner(&doc, &provider).unwrap());
}
