//! Ownership Validation Rule testing

use catalyst_types::{catalyst_id::role_index::RoleId, uuid::UuidV7};
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    validator::rules::utils::create_dummy_key_pair,
};

#[test_case(
    |_| {
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => (true, true) ;
    "First Version Catalyst Signed Document has only one author"
)]
#[test_case(
    |provider| {
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => (true, true) ;
   "Latest Version Catalyst Signed Document has the same author as the first version"
)]
#[test_case(
    |provider| {
        let (a_sk, _, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, _, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Collaborators(vec![c_kid.clone()].into()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => (false, true) ;
   "Latest Version Catalyst Signed Document signed by first author and one collaborator"
)]
#[test_case(
    |provider| {
        let (a_sk, _, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, _, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Collaborators(vec![c_kid.clone()].into()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => (false, true) ;
   "Latest Version Catalyst Signed Document signed by one collaborator"
)]
#[test_case(
    |_| {
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build()
    } => (false, false) ;
    "First Version Unsigned Catalyst Document"
)]
#[test_case(
    |_| {
        let (sk1, _, kid1) = create_dummy_key_pair(RoleId::Role0);
        let (sk2, _, kid2) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk1.sign(&m).to_vec(), kid1.clone())
            .unwrap()
            .add_signature(|m| sk2.sign(&m).to_vec(), kid2.clone())
            .unwrap()
            .build()
    } => (false, false) ;
    "First Version Catalyst Signed Document two authors"
)]
#[test_case(
    |provider| {
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0);
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
    } => (false, false) ;
    "Latest Catalyst Signed Document has a different author from the first version"
)]
#[test_case(
    |provider| {
        let (a_sk, _, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, _, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => (false, false) ;
   "Latest Version Catalyst Signed Document signed by first author and not added collaborator"
)]
#[test_case(
    |provider| {
        let (a_sk, _, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c1_sk, _, c1_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Collaborators(vec![c1_kid.clone()].into()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        let (c2_sk, _, c2_kid) = create_dummy_key_pair(RoleId::Role0);
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .add_signature(|m| c1_sk.sign(&m).to_vec(), c1_kid.clone())
            .unwrap()
            .add_signature(|m| c2_sk.sign(&m).to_vec(), c2_kid.clone())
            .unwrap()
            .build()
    } => (false, false) ;
   "Latest Version Catalyst Signed Document signed by first author and one collaborator and one unknown collaborator"
)]
#[tokio::test]
async fn ownership_without_collaborators_test(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> (bool, bool) {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider);

    let without_collaborators = DocumentOwnershipRule {
        allow_collaborators: false,
    }
    .check(&doc, &provider)
    .await
    .unwrap();
    let allowed_collaborators = DocumentOwnershipRule {
        allow_collaborators: true,
    }
    .check(&doc, &provider)
    .await
    .unwrap();
    println!("{:?}", doc.report());
    (without_collaborators, allowed_collaborators)
}
