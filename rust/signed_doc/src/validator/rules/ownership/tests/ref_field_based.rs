//! Ownership Validation Rule tests for `DocumentOwnershipRule::RefFieldBased`
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
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        let id = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![doc.doc_ref().unwrap()].into()
            ))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build()
    } => true ;
   "Latest Version Catalyst Signed Document signed by first author of the referenced doc"
)]
#[test_case(
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Collaborators(vec![c_kid.clone()].into()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        let id = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![doc.doc_ref().unwrap()].into()
            ))
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => true ;
   "Latest Version Catalyst Signed Document signed by one collaborator of the referenced doc"
)]
#[test_case(
    |_| {
        let id = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build()
    } => false ;
    "First Version Unsigned Catalyst Document"
)]
#[test_case(
    |_| {
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => false ;
    "First Version Catalyst Signed Document has only one author, missing ref field"
)]
#[test_case(
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(&doc).unwrap();

        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![doc.doc_ref().unwrap()].into()
            ))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build()
    } => false ;
   "Latest Version Catalyst Signed Document signed by other author"
)]
fn ownership_test(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider);

    DocumentOwnershipRule::RefFieldBased
        .check_inner(&doc, &provider)
        .unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
