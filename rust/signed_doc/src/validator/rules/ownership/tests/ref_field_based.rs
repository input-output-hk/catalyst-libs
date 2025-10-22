//! Ownership Validation Rule tests for `DocumentOwnershipRule::RefFieldBased`
//! scenario

use catalyst_types::{catalyst_id::role_index::RoleId, uuid::UuidV7};
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::{
    builder::tests::Builder,
    metadata::SupportedField,
    providers::tests::TestCatalystProvider,
    validator::rules::{utils::create_dummy_key_pair, DocumentOwnershipRule},
    CatalystSignedDocument, DocLocator, DocumentRef,
};

#[test_case(
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![
                    DocumentRef::new(
                        doc.doc_id().unwrap(),
                        doc.doc_ver().unwrap(),
                        DocLocator::default()
                    )
                ].into()
            ))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build()
    } => true ;
   "Latest Version Catalyst Signed Document signed by first author of the refenced doc"
)]
#[test_case(
    |provider| {
        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let (c_sk, c_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Collaborators(vec![c_kid.clone()].into()))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![
                    DocumentRef::new(
                        doc.doc_id().unwrap(),
                        doc.doc_ver().unwrap(),
                        DocLocator::default()
                    )
                ].into()
            ))
            .add_signature(|m| c_sk.sign(&m).to_vec(), c_kid.clone())
            .unwrap()
            .build()
    } => true ;
   "Latest Version Catalyst Signed Document signed by one collaborator of the refenced doc"
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
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
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
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build();
        provider.add_document(None, &doc).unwrap();

        let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
        let id = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .with_metadata_field(SupportedField::Ref(
                vec![
                    DocumentRef::new(
                        doc.doc_id().unwrap(),
                        doc.doc_ver().unwrap(),
                        DocLocator::default()
                    )
                ].into()
            ))
            .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
            .unwrap()
            .build()
    } => false ;
   "Latest Version Catalyst Signed Document signed by other author"
)]
#[tokio::test]
async fn ownership_test(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider);

    let res = DocumentOwnershipRule::RefFieldBased
        .check(&doc, &provider)
        .await
        .unwrap();
    println!("{:?}", doc.report());
    res
}
