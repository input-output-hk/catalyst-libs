//! Integration test for the `catalyst_signed_doc` crate.

use std::str::FromStr;

use catalyst_signed_doc::*;

mod common;

#[test]
fn catalyst_signed_doc_cbor_roundtrip_test() {
    let (uuid_v7, uuid_v4, metadata_fields) = common::test_metadata();
    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .build();

    assert!(!doc.problem_report().is_problematic());

    let bytes: Vec<u8> = doc.try_into().unwrap();
    let decoded: CatalystSignedDocument = bytes.as_slice().try_into().unwrap();

    assert_eq!(decoded.doc_type().unwrap(), uuid_v4);
    assert_eq!(decoded.doc_id().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_ver().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_content().decoded_bytes().unwrap(), &content);
    // TODO: after this test will be moved as a crate integration test, enable this
    // assertion assert_eq!(decoded.doc_meta(), metadata_fields.extra());
}

#[tokio::test]
async fn signature_verification_test() {
    let mut csprng = rand::rngs::OsRng;
    let sk = ed25519_dalek::SigningKey::generate(&mut csprng);
    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();
    let pk = sk.verifying_key();

    let kid_str = format!(
        "id.catalyst://cardano/{}/0/0",
        base64_url::encode(pk.as_bytes())
    );

    let kid = IdUri::from_str(&kid_str).unwrap();
    let (_, _, metadata) = common::test_metadata();
    let signed_doc = Builder::new()
        .with_decoded_content(content)
        .with_json_metadata(metadata)
        .unwrap()
        .add_signature(sk.to_bytes(), kid.clone())
        .unwrap()
        .build();
    assert!(!signed_doc.problem_report().is_problematic());

    assert!(validator::validate_signatures(
        &signed_doc,
        &common::Provider(Err(anyhow::anyhow!("some error")))
    )
    .await
    .is_err());
    assert!(
        validator::validate_signatures(&signed_doc, &common::Provider(Ok(Some(pk))))
            .await
            .unwrap()
    );
    assert!(
        !validator::validate_signatures(&signed_doc, &common::Provider(Ok(None)))
            .await
            .unwrap()
    );
}
