//! Integration test for comment document validation part.

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn test_valid_comment_doc() {
    let (proposal_doc, proposal_doc_id) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "template": {
          "id": template_doc_id
        },
        "ref": {
            "id": proposal_doc_id
        }
    })))
    .unwrap();

    let provider = common::DummyCatSignDocProvider(From::from([
        (template_doc_id.into(), template_doc),
        (proposal_doc_id.into(), proposal_doc),
    ]));

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_comment_doc_with_reply() {
    let empty_json = serde_json::to_vec(&serde_json::json!({})).unwrap();

    let (proposal_doc, proposal_doc_id) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let comment_doc_id = UuidV7::new();
    let comment_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
            "content-type": ContentType::Json.to_string(),
            "template": { "id": comment_doc_id.to_string() },
            "ref": {
                "id": proposal_doc_id
            },
        }))
        .unwrap()
        .with_decoded_content(empty_json.clone())
        .build();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "template": {
          "id": template_doc_id
        },
        "ref": {
            "id": proposal_doc_id
        },
        "reply": {
            "id": comment_doc_id,
            "ver": uuid_v7
        }
    })))
    .unwrap();

    let provider = common::DummyCatSignDocProvider(From::from([
        (template_doc_id.into(), template_doc),
        (proposal_doc_id.into(), proposal_doc),
        (comment_doc_id.into(), comment_doc),
    ]));

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_invalid_comment_doc() {
    let (proposal_doc, proposal_doc_id) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "template": {
          "id": template_doc_id
        },
        // without ref
        "ref": serde_json::Value::Null
    })))
    .unwrap();

    let provider = common::DummyCatSignDocProvider(From::from([
        (template_doc_id.into(), template_doc),
        (proposal_doc_id.into(), proposal_doc),
    ]));

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
