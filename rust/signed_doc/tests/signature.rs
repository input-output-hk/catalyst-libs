//! Integration test for signature validation part.

use std::collections::HashMap;

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn single_signature_validation_test() {
    let (signed_doc, pk, kid) = common::create_dummy_signed_doc(None).unwrap();
    assert!(!signed_doc.problem_report().is_problematic());

    // case: has key
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(From::from([(kid, pk)]))
    )
    .await
    .is_ok_and(|v| v));

    // case: empty provider
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(HashMap::default())
    )
    .await
    .is_ok_and(|v| !v));
}

#[tokio::test]
async fn multiple_signatures_validation_test() {
    let (sk1, pk1, kid1) = common::create_dummy_key_pair().unwrap();
    let (sk2, pk2, kid2) = common::create_dummy_key_pair().unwrap();
    let (sk3, pk3, kid3) = common::create_dummy_key_pair().unwrap();
    let (_, pk_n, kid_n) = common::create_dummy_key_pair().unwrap();

    let signed_doc = Builder::new()
        .with_decoded_content(serde_json::to_vec(&serde_json::Value::Null).unwrap())
        .with_json_metadata(common::test_metadata().2)
        .unwrap()
        .add_signature(sk1.to_bytes(), kid1.clone())
        .unwrap()
        .add_signature(sk2.to_bytes(), kid2.clone())
        .unwrap()
        .add_signature(sk3.to_bytes(), kid3.clone())
        .unwrap()
        .build();

    assert!(!signed_doc.problem_report().is_problematic());

    // case: all signatures valid
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(From::from([
            (kid1.clone(), pk1),
            (kid2.clone(), pk2),
            (kid3.clone(), pk3)
        ]))
    )
    .await
    .is_ok_and(|v| v));

    // case: partially available signatures
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(From::from([(kid1.clone(), pk1), (kid2.clone(), pk2)]))
    )
    .await
    .is_ok_and(|v| !v));

    // caes: with unrecognized provider
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(From::from([(kid_n.clone(), pk_n)]))
    )
    .await
    .is_ok_and(|v| !v));

    // case: no valid signatures available
    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(HashMap::default())
    )
    .await
    .is_ok_and(|v| !v));
}
