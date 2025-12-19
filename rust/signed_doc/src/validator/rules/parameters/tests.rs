use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_doc_ref,
};

#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => true
    ;
    "valid reference to the valid parameters document"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let common_parameter_field: DocumentRefs = vec![parameter_doc.doc_ref().unwrap()]
                .into();
        let template_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
            .build();
        provider.add_document(&template_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field))
            .build()
    }
    => true
    ;
    "valid reference to the valid parameters document, with valid template field"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![create_dummy_doc_ref()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing template doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let template_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build();
        provider.add_document(&template_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_doc.doc_ref().unwrap()]
                .into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing parameters field in template doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let template_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(
                vec![create_dummy_doc_ref()].into()
            ))
            .build();
        provider.add_document(&template_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_doc.doc_ref().unwrap()]
                .into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with different parameters field in template doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let common_parameter_field: DocumentRefs = vec![parameter_doc.doc_ref().unwrap()]
                .into();
        let replied_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
            .build();
        provider.add_document(&replied_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![replied_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field))
            .build()
    }
    => true
    ;
    "valid reference to the valid parameters document, with valid reply field"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![create_dummy_doc_ref()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing reply doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let reply_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build();
        provider.add_document(&reply_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![reply_doc.doc_ref().unwrap()]
                .into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing parameters field in replied doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let reply_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(
                vec![create_dummy_doc_ref()].into()
            ))
            .build();
        provider.add_document(&reply_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Reply(
                vec![reply_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with different parameters field in reply doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let common_parameter_field: DocumentRefs = vec![parameter_doc.doc_ref().unwrap()]
                .into();
        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
            .build();
        provider.add_document(&ref_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![ref_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(common_parameter_field))
            .build()
    }
    => true
    ;
    "valid reference to the valid parameters document, with valid ref field"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![create_dummy_doc_ref()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing ref doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build();
        provider.add_document(&ref_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![ref_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with missing parameters field in ref doc"
)]
#[test_case(
    |exp_param_types, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        let ref_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(
                vec![create_dummy_doc_ref()].into()
            ))
            .build();
        provider.add_document(&ref_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![ref_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "valid reference to the valid parameters document, with different parameters field in ref doc"
)]
#[test_case(
    |_, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the invalid parameters document, wrong parameters type field value"
)]
#[test_case(
    |_, provider| {
        let parameter_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build();
        provider.add_document(&parameter_doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Parameters(
                vec![parameter_doc.doc_ref().unwrap()].into(),
            ))
            .build()
    }
    => false
    ;
    "valid reference to the invalid parameters document, missing type field"
)]
#[test_case(
    |_, _| {
        Builder::new()
            .with_metadata_field(SupportedField::Parameters(
                vec![create_dummy_doc_ref()].into(),
            ))
            .build()
    }
    => false
    ;
    "reference to the not known document"
)]
#[test_case(
    |exp_param_types, provider| {
        let p1_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
        provider.add_document(&p1_doc).unwrap();


        let p2_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[1].clone()))
            .with_metadata_field(SupportedField::Parameters(
                vec![p1_doc.doc_ref().unwrap()].into()
            ))
            .build();
        provider.add_document(&p2_doc).unwrap();

        let t2_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(
                vec![p1_doc.doc_ref().unwrap()].into()
            ))
            .build();
        provider.add_document(&t2_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![t2_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![p2_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => true
    ;
    "valid reference to valid one-level recursion parameters field"
)]
#[test_case(
    |exp_param_types, provider| {
        let p1_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
        provider.add_document(&p1_doc).unwrap();


        let p2_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_param_types[1].clone()))
            .build();
        provider.add_document(&p2_doc).unwrap();

        let t2_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Parameters(
                vec![p1_doc.doc_ref().unwrap()].into()
            ))
            .build();
        provider.add_document(&t2_doc).unwrap();


        Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![t2_doc.doc_ref().unwrap()].into()
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![p2_doc.doc_ref().unwrap()].into()
            ))
            .build()
    }
    => false
    ;
    "reference to non-linked parameters field"
)]
#[tokio::test]
fn parameter_specified_test(
    doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let exp_param_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

    let doc = doc_gen(&exp_param_types, &mut provider);

    let non_optional_res = ParametersRule::Specified {
        allowed_type: exp_param_types.to_vec(),
        optional: false,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    let optional_res = ParametersRule::Specified {
        allowed_type: exp_param_types.to_vec(),
        optional: true,
    }
    .check(&doc, &provider)
    .await
    .unwrap();

    println!("{:?}", doc.report());
    assert_eq!(non_optional_res, optional_res);
    non_optional_res
}

#[tokio::test]
fn parameters_specified_optional_test() {
    let provider = TestCatalystProvider::default();
    let rule = ParametersRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        optional: true,
    };

    let doc = Builder::new().build();
    assert!(rule.check(&doc, &provider).await.unwrap());

    let provider = TestCatalystProvider::default();
    let rule = ParametersRule::Specified {
        allowed_type: vec![UuidV4::new().into()],
        optional: false,
    };

    let doc = Builder::new().build();
    assert!(!rule.check(&doc, &provider).await.unwrap());
}

#[tokio::test]
fn parameters_rule_not_specified_test() {
    let rule = ParametersRule::NotSpecified;
    let provider = TestCatalystProvider::default();

    let doc = Builder::new().build();
    assert!(rule.check(&doc, &provider).await.unwrap());

    let doc = Builder::new()
        .with_metadata_field(SupportedField::Parameters(
            vec![create_dummy_doc_ref()].into(),
        ))
        .build();
    assert!(!rule.check(&doc, &provider).await.unwrap());
}
