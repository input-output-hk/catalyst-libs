//! Validation tests.

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn test_valid_proposal_doc() {
    let uuid_v7 = UuidV7::new();

    let (doc, _) = common::get_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": serde_json::Value::Null,
        "reply": serde_json::Value::Null,
        "template": {
          "id": doc_types::PROPOSAL_TEMPLATE_UUID_TYPE
        },
        "section": serde_json::Value::Null,
        "collabs": serde_json::Value::Array(vec![]),
        "campaign_id": serde_json::Value::Null,
        "election_id":  serde_json::Value::Null,
        "brand_id":  serde_json::Value::Null,
        "category_id": serde_json::Value::Null,
    })));

    let provider = common::DummyCatSignDocProvider::default();

    let result = validator::validate(&doc, &provider).await.unwrap();

    println!("{:?}", doc.problem_report());

    assert!(result);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {}
