//! Integration test for the `catalyst_signed_doc` crate.

use std::str::FromStr;

use catalyst_signed_doc::*;

mod decoding;
mod signature;
mod validation;

fn test_metadata() -> (UuidV7, UuidV4, serde_json::Value) {
    let alg = Algorithm::EdDSA;
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let section = "$".to_string();
    let collabs = vec!["Alex1".to_string(), "Alex2".to_string()];
    let content_type = ContentType::Json;
    let content_encoding = ContentEncoding::Brotli;

    let metadata_fields = serde_json::json!({
        "alg": alg.to_string(),
        "content-type": content_type.to_string(),
        "content-encoding": content_encoding.to_string(),
        "type": uuid_v4.to_string(),
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": {"id": uuid_v7.to_string()},
        "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
        "template": {"id": uuid_v7.to_string()},
        "section": section,
        "collabs": collabs,
        "campaign_id": {"id": uuid_v7.to_string()},
        "election_id":  uuid_v4.to_string(),
        "brand_id":  {"id": uuid_v7.to_string()},
        "category_id": {"id": uuid_v7.to_string()},
    });
    (uuid_v7, uuid_v4, metadata_fields)
}

struct Provider(anyhow::Result<Option<ed25519_dalek::VerifyingKey>>);
impl providers::VerifyingKeyProvider for Provider {
    async fn try_get_key(
        &self, _kid: &IdUri,
    ) -> anyhow::Result<Option<ed25519_dalek::VerifyingKey>> {
        let res = self.0.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(*res)
    }
}

#[test]
fn catalyst_signed_doc_cbor_roundtrip_test() {
    let (uuid_v7, uuid_v4, metadata_fields) = test_metadata();
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
    let (_, _, metadata) = test_metadata();
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
        &Provider(Err(anyhow::anyhow!("some error")))
    )
    .await
    .is_err());
    assert!(
        validator::validate_signatures(&signed_doc, &Provider(Ok(Some(pk))))
            .await
            .unwrap()
    );
    assert!(
        !validator::validate_signatures(&signed_doc, &Provider(Ok(None)))
            .await
            .unwrap()
    );
}
