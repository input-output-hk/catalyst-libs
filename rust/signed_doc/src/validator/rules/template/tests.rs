use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_doc_ref,
};

#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => true
    ;
    "content is complied with the referenced template json schema"
)]
#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "missing template field"
)]
#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .build()
    }
    => false
    ;
    "missing content"
)]
#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(vec![1, 2, 3,])
            .build()
    }
    => false
    ;
    "content is not valid JSON"
)]
#[test_case(
    |_, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let template_ref = create_dummy_doc_ref();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(*template_ref.id()))
            .with_metadata_field(SupportedField::Ver(*template_ref.ver()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "wrong 'type' in the referenced template document"
)]
#[test_case(
    |_, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "missing 'type' field in the referenced template document"
)]
#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "missing 'content-type' field in the referenced template document'"
)]
#[test_case(
    |allowed_type, provider| {
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "missing content in the referenced template document"
)]
#[test_case(
    |allowed_type, provider| {
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(vec![1,2 ,3])
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "content is not a JSON schema in the referenced template document"
)]
#[test_case(
    |_, _| {
        let template_ref = create_dummy_doc_ref();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "referencing to unknown document"
)]
fn template_specified_test(
    doc_gen: impl FnOnce(DocType, &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let allowed_type: DocType = UuidV4::new().into();

    let doc = doc_gen(allowed_type.clone(), &mut provider);

    TemplateRule::Specified { allowed_type }
        .check_inner(&doc, &provider)
        .unwrap()
}

#[test_case(
    |_, _| {
        Builder::new()
            .build()
    }
    => true
    ;
    "missing 'template' field"
)]
#[test_case(
    |allowed_type, provider| {
        let json_schema = serde_json::to_vec(&serde_json::json!({})).unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(allowed_type))
            .with_metadata_field(SupportedField::ContentType(ContentType::SchemaJson))
            .with_content(json_schema)
            .build();
        provider.add_document(&doc).unwrap();
        let template_ref = doc.doc_ref().unwrap();

        let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Template(
                vec![template_ref].into(),
            ))
            .with_content(json_content)
            .build()
    }
    => false
    ;
    "content is complied with the referenced template json schema for non specified 'template' field"
)]
fn reply_rule_not_specified_test(
    doc_gen: impl FnOnce(DocType, &mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let allowed_type: DocType = UuidV4::new().into();
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(allowed_type, &mut provider);
    TemplateRule::NotSpecified
        .check_inner(&doc, &provider)
        .unwrap()
}
