//! Ownership Validation Rule testing

use catalyst_types::{catalyst_id::role_index::RoleId, uuid::UuidV7};
use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    CatalystSignedDocument, builder::tests::Builder, metadata::SupportedField,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair,
    validator::rules::DocumentOwnershipRule,
};

mod collaborators_field_based;
mod ref_field_based;
mod without_collaborators;

#[test]
fn empty_provider_test() {
    let provider = TestCatalystProvider::default();

    let doc = make_document();
    DocumentOwnershipRule::OriginalAuthor
        .check_inner(&doc, &provider)
        .unwrap();
    assert!(doc.report().is_problematic());
    let report = format!("{:?}", doc.report());
    assert!(
        report.contains("Cannot find a first version of the referenced document"),
        "{report}"
    );

    let doc = make_document();
    DocumentOwnershipRule::RefFieldBased
        .check_inner(&doc, &provider)
        .unwrap();
    assert!(doc.report().is_problematic());
    let report = format!("{:?}", doc.report());
    assert!(report.contains("Document ownership validation"), "{report}");

    let doc = make_document();
    DocumentOwnershipRule::CollaboratorsFieldBased
        .check_inner(&doc, &provider)
        .unwrap();
    assert!(doc.report().is_problematic());
    let report = format!("{:?}", doc.report());
    assert!(
        report.contains("Cannot find a first version of the referenced document"),
        "{report}"
    );
}

/// Creates a document for testing.
fn make_document() -> CatalystSignedDocument {
    let (a_sk, a_kid) = create_dummy_key_pair(RoleId::Role0);
    let id = UuidV7::new();
    let ver = UuidV7::new();
    Builder::with_required_fields()
        .with_metadata_field(SupportedField::Id(id))
        .with_metadata_field(SupportedField::Ver(ver))
        .add_signature(|m| a_sk.sign(&m).to_vec(), a_kid.clone())
        .unwrap()
        .build()
}
