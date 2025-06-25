//! Integration test for signature validation part.

use catalyst_signed_doc::{providers::tests::TestVerifyingKeyProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

fn metadata() -> serde_json::Value {
    serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": UuidV4::new(),
        "id":  UuidV7::new(),
        "ver":  UuidV7::new(),
        "ref": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
        "reply": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
        "template": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
        "section": "$",
        "collabs": vec!["Alex1", "Alex2"],
        "parameters": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
    })
}

#[tokio::test]
async fn single_signature_validation_test() {
    let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    let signed_doc = Builder::new()
        .with_json_metadata(metadata())
        .unwrap()
        .with_json_content(&serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    assert!(!signed_doc.problem_report().is_problematic());

    // case: has key
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid.clone(), pk);
    assert!(
        validator::validate_signatures(&signed_doc, &provider)
            .await
            .unwrap(),
        "{:?}",
        signed_doc.problem_report()
    );

    // case: empty provider
    assert!(
        !validator::validate_signatures(&signed_doc, &TestVerifyingKeyProvider::default())
            .await
            .unwrap()
    );

    // case: signed with different key
    let (another_sk, ..) = create_dummy_key_pair(RoleId::Role0).unwrap();
    let invalid_doc = signed_doc
        .into_builder()
        .add_signature(|m| another_sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();
    assert!(!validator::validate_signatures(&invalid_doc, &provider)
        .await
        .unwrap());

    // case: missing signatures
    let unsigned_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": UuidV4::new(),
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();
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
        .with_json_metadata(metadata())
        .unwrap()
        .with_json_content(&serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk1.sign(&m).to_vec(), kid1.clone())
        .unwrap()
        .add_signature(|m| sk2.sign(&m).to_vec(), kid2.clone())
        .unwrap()
        .add_signature(|m| sk3.sign(&m).to_vec(), kid3.clone())
        .unwrap()
        .build()
        .unwrap();

    assert!(!signed_doc.problem_report().is_problematic());

    // case: all signatures valid
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid1.clone(), pk1);
    provider.add_pk(kid2.clone(), pk2);
    provider.add_pk(kid3.clone(), pk3);
    assert!(validator::validate_signatures(&signed_doc, &provider)
        .await
        .unwrap());

    // case: partially available signatures
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid1.clone(), pk1);
    provider.add_pk(kid2.clone(), pk2);
    assert!(!validator::validate_signatures(&signed_doc, &provider)
        .await
        .unwrap());

    // case: with unrecognized provider
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid_n.clone(), pk_n);
    assert!(!validator::validate_signatures(&signed_doc, &provider)
        .await
        .unwrap());

    // case: no valid signatures available
    assert!(
        !validator::validate_signatures(&signed_doc, &TestVerifyingKeyProvider::default())
            .await
            .unwrap()
    );
}
