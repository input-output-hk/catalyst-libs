//! Integration test for signature validation part.

use catalyst_signed_doc::{providers::tests::TestVerifyingKeyProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use common::test_metadata;
use ed25519_dalek::ed25519::signature::Signer;

mod common;

#[tokio::test]
async fn single_signature_validation_test() {
    let (_, _, metadata) = test_metadata();
    let (signed_doc, pk, kid) = common::create_dummy_signed_doc(
        metadata,
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();
    assert!(!signed_doc.problem_report().is_problematic());

    // case: has key
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid.clone(), pk);
    assert!(validator::validate_signatures(&signed_doc, &provider)
        .await
        .unwrap());

    // case: empty provider
    assert!(
        !validator::validate_signatures(&signed_doc, &TestVerifyingKeyProvider::default())
            .await
            .unwrap()
    );

    // case: missing signatures
    let (unsigned_doc, ..) = common::create_dummy_doc(UuidV4::new().into()).unwrap();
    assert!(!validator::validate_signatures(&unsigned_doc, &provider)
        .await
        .unwrap());
}

#[tokio::test]
async fn multiple_signatures_validation_test() {
    let (sk1, pk1, kid1) = common::create_dummy_key_pair(RoleId::Role0).unwrap();
    let (sk2, pk2, kid2) = common::create_dummy_key_pair(RoleId::Role0).unwrap();
    let (sk3, pk3, kid3) = common::create_dummy_key_pair(RoleId::Role0).unwrap();
    let (_, pk_n, kid_n) = common::create_dummy_key_pair(RoleId::Role0).unwrap();

    let signed_doc = Builder::new()
        .with_decoded_content(serde_json::to_vec(&serde_json::Value::Null).unwrap())
        .with_json_metadata(common::test_metadata().2)
        .unwrap()
        .add_signature(|m| sk1.sign(&m).to_vec(), kid1.clone())
        .unwrap()
        .add_signature(|m| sk2.sign(&m).to_vec(), kid2.clone())
        .unwrap()
        .add_signature(|m| sk3.sign(&m).to_vec(), kid3.clone())
        .unwrap()
        .build();

    assert!(!signed_doc.problem_report().is_problematic());

    // case: all signatures valid
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid1.clone(), pk1);
    provider.add_pk(kid2.clone(), pk2);
    provider.add_pk(kid3.clone(), pk3);
    assert!(validator::validate_signatures(&signed_doc, &provider)
        .await
        .is_ok_and(|v| v));

    // case: partially available signatures
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid1.clone(), pk1);
    provider.add_pk(kid2.clone(), pk2);
    assert!(validator::validate_signatures(&signed_doc, &provider)
        .await
        .is_ok_and(|v| !v));

    // case: with unrecognized provider
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid_n.clone(), pk_n);
    assert!(validator::validate_signatures(&signed_doc, &provider)
        .await
        .is_ok_and(|v| !v));

    // case: no valid signatures available
    assert!(
        validator::validate_signatures(&signed_doc, &TestVerifyingKeyProvider::default())
            .await
            .is_ok_and(|v| !v)
    );
}
