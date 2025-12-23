//! Ownership Validation Rule tests for `DocumentOwnershipRule::OriginalAuthor`
//! scenario

use catalyst_types::{catalyst_id::role_index::RoleId, uuid::UuidV7};
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::{
    CatalystSignedDocument, builder::tests::Builder, metadata::SupportedField,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair,
    validator::rules::DocumentOwnershipRule,
};

#[test_case(
    |_| {
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => true ;
    "First Version Catalyst Signed Document has only one author"
)]
#[test_case(
    |provider| {
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => true ;
   "Latest Version Catalyst Signed Document has the same author as the first version"
)]
#[test_case(
    |_| {
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build()
    } => false ;
    "First Version Unsigned Catalyst Document"
)]
#[test_case(
    |_| {
        let (sk1, kid1) = create_dummy_key_pair(RoleId::Role0);
        let (sk2, kid2) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk1.sign(&m).to_vec(), kid1.clone())
            .unwrap()
            .add_signature(|m| sk2.sign(&m).to_vec(), kid2.clone())
            .unwrap()
            .build()
    } => false ;
    "First Version Catalyst Signed Document two authors"
)]
#[test_case(
    |provider| {
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => false ;
    "Latest Catalyst Signed Document has a different author from the first version"
)]
#[test_case(
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => false ;
   "Latest Version Catalyst Signed Document signed by first author and not added collaborator"
)]
fn ownership_test(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider);

    let res = DocumentOwnershipRule::OriginalAuthor
        .check_inner(&doc, &provider)
        .unwrap();
    println!("{:?}", doc.report());
    res
}
