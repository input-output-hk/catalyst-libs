use catalyst_types::uuid::UuidV4;
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    uuid::UuidV7,
};

#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let doc_type = UuidV4::new();
        let id = UuidV7::new();

        let first_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Type(doc_type.into()))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .with_metadata_field(SupportedField::Type(doc_type.into()))
            .build()
    }
    => true;
    "valid"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |_| {
        let id = UuidV7::new();
        let ver = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .build()
    }
    => false;
    "missing first version document"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let doc_type = UuidV4::new();

        let id = UuidV7::new();
        let first_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Type(doc_type.into()))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .build()
    }
    => false;
    "missing `type` field"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let doc_type = UuidV4::new();
        let id = UuidV7::new();

        let first_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .with_metadata_field(SupportedField::Type(doc_type.into()))
            .build()
    }
    => false;
    "missing `type` field for the latest known document"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {

        let id = UuidV7::new();
        let first_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build()
    }
    => false;
    "diverge `type` field with the latest known document"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .build()
    }
    => false;
    "missing `ver` field"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build()
    }
    => false;
    "missing `id` field"
)]
fn type_test(doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider);

    TypeRule::check_inner(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
