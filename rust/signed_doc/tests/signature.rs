//! Integration test for signature validation part.

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn signature_verification_test() {
    let (signed_doc, pk) = common::create_dummy_signed_doc(None).unwrap();
    assert!(!signed_doc.problem_report().is_problematic());

    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Err(anyhow::anyhow!("some error")))
    )
    .await
    .is_err());

    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Ok(Some(pk)))
    )
    .await
    .unwrap());

    assert!(!validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Ok(None))
    )
    .await
    .unwrap());
}
