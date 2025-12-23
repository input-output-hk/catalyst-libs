//! Ownership Validation Rule testing

use catalyst_types::{catalyst_id::role_index::RoleId, uuid::UuidV7};
use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_key_pair, validator::rules::DocumentOwnershipRule,
};

mod collaborators_field_based;
mod ref_field_based;
mod without_collaborators;

#[test]
fn empty_provider_test() {
    let provider = TestCatalystProvider::default();

    let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
    let id = UuidV7::new();
    let ver = UuidV7::new();
    let doc = Builder::new()
        .with_metadata_field(SupportedField::Id(id))
        .with_metadata_field(SupportedField::Ver(ver))
        .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
        .unwrap()
        .build();

    let result = DocumentOwnershipRule::OriginalAuthor.check_inner(&doc, &provider);
    assert!(matches!(result, Ok(false)));
    let result = DocumentOwnershipRule::RefFieldBased.check_inner(&doc, &provider);
    assert!(matches!(result, Ok(false)));
    let result = DocumentOwnershipRule::CollaboratorsFieldBased.check_inner(&doc, &provider);
    assert!(matches!(result, Ok(false)));
}
