//! Integration test for COSE decoding part.

use catalyst_signed_doc::{providers::tests::TestVerifyingKeyProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use common::create_dummy_key_pair;
use coset::TaggedCborSerializable;
use ed25519_dalek::ed25519::signature::Signer;

mod common;

#[test]
fn catalyst_signed_doc_cbor_roundtrip_kid_as_id_test() {
    let (_, _, metadata_fields) = common::test_metadata();
    let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
    // transform Catalyst ID URI form to the ID form
    let kid = kid.as_id();

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();

    assert!(doc.problem_report().is_problematic());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn catalyst_signed_doc_parameters_aliases_test() {
    let (_, _, metadata_fields) = common::test_metadata();
    let (sk, pk, kid) = common::create_dummy_key_pair(RoleId::Role0).unwrap();
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid.clone(), pk);

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .build();
    assert!(!doc.problem_report().is_problematic());

    let parameters_val = doc.doc_meta().parameters().unwrap();
    let parameters_val_cbor: coset::cbor::Value = parameters_val.try_into().unwrap();
    // replace parameters with the alias values `category_id`, `brand_id`, `campaign_id`.
    let bytes: Vec<u8> = doc.try_into().unwrap();
    let mut cose = coset::CoseSign::from_tagged_slice(bytes.as_slice()).unwrap();
    cose.protected.original_data = None;
    cose.protected
        .header
        .rest
        .retain(|(l, _)| l != &coset::Label::Text("parameters".to_string()));

    let doc: CatalystSignedDocument = cose
        .clone()
        .to_tagged_vec()
        .unwrap()
        .as_slice()
        .try_into()
        .unwrap();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_none());

    // case: `category_id`.
    let mut cose_with_category_id = cose.clone();
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("category_id".to_string()),
        parameters_val_cbor.clone(),
    ));

    let cbor_bytes = cose_with_category_id.to_tagged_vec().unwrap();
    let doc: CatalystSignedDocument = cbor_bytes.as_slice().try_into().unwrap();
    let doc = doc
        .into_builder()
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());
    assert!(validator::validate_signatures(&doc, &provider)
        .await
        .unwrap());

    // case: `brand_id`.
    let mut cose_with_brand_id = cose.clone();
    cose_with_brand_id.protected.header.rest.push((
        coset::Label::Text("brand_id".to_string()),
        parameters_val_cbor.clone(),
    ));

    let cbor_bytes = cose_with_brand_id.to_tagged_vec().unwrap();
    let doc: CatalystSignedDocument = cbor_bytes.as_slice().try_into().unwrap();
    let doc = doc
        .into_builder()
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());
    assert!(validator::validate_signatures(&doc, &provider)
        .await
        .unwrap());

    // case: `campaign_id`.
    let mut cose_with_campaign_id = cose.clone();
    cose_with_campaign_id.protected.header.rest.push((
        coset::Label::Text("campaign_id".to_string()),
        parameters_val_cbor.clone(),
    ));

    let cbor_bytes = cose_with_campaign_id.to_tagged_vec().unwrap();
    let doc: CatalystSignedDocument = cbor_bytes.as_slice().try_into().unwrap();
    let doc = doc
        .into_builder()
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());
    assert!(validator::validate_signatures(&doc, &provider)
        .await
        .unwrap());

    // `parameters` value along with its aliases are not allowed to be present at the
    let mut cose_with_category_id = cose.clone();
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("parameters".to_string()),
        parameters_val_cbor.clone(),
    ));
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("category_id".to_string()),
        parameters_val_cbor.clone(),
    ));
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("brand_id".to_string()),
        parameters_val_cbor.clone(),
    ));
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("campaign_id".to_string()),
        parameters_val_cbor.clone(),
    ));

    let doc: CatalystSignedDocument = cose_with_category_id
        .to_tagged_vec()
        .unwrap()
        .as_slice()
        .try_into()
        .unwrap();
    assert!(doc.problem_report().is_problematic());
}

struct TestCase {
    name: &'static str,
    bytes_gen: Box<dyn Fn() -> Vec<u8>>,
    // If the provided bytes can be even decoded without error (valid COSE or not).
    // If set to `false` all futher checks will not even happen.
    can_decode: bool,
    // If the decoded doc is a valid `CatalystSignedDocument`, underlying problem report is empty.
    valid_doc: bool,
    post_checks: Option<Box<dyn Fn(&CatalystSignedDocument) -> bool>>,
}

fn decoding_empty_bytes_case() -> TestCase {
    TestCase {
        name: "Decoding empty bytes",
        bytes_gen: Box::new(|| vec![]),
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_all_fields_case() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    TestCase {
        name: "Catalyst Signed Doc with ALL defined metadata fields and signatures",
        bytes_gen: Box::new({
            let kid = kid.clone();
            move || {
                Builder::new()
                    .with_json_metadata(serde_json::json!({
                        "content-type": ContentType::Json.to_string(),
                        "content-encoding": ContentEncoding::Brotli.to_string(),
                        "type": uuid_v4.to_string(),
                        "id": uuid_v7.to_string(),
                        "ver": uuid_v7.to_string(),
                        "ref": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
                        "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
                        "template": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
                        "section": "$".to_string(),
                        "collabs": vec!["Alex1".to_string(), "Alex2".to_string()],
                        "parameters": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
                    }))
                    .unwrap()
                    .with_decoded_content(serde_json::to_vec(&serde_json::Value::Null).unwrap())
                    .add_signature(|m| sk.sign(&m).to_vec(), &kid)
                    .unwrap()
                    .build()
                    .try_into()
                    .unwrap()
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                (doc.doc_type().unwrap() == uuid_v4)
                    && (doc.doc_id().unwrap() == uuid_v7)
                    && (doc.doc_ver().unwrap() == uuid_v7)
                    && (doc.doc_content_type().unwrap() == ContentType::Json)
                    && (doc.doc_content_encoding().unwrap() == ContentEncoding::Brotli)
                    && (doc.doc_meta().doc_ref().unwrap()
                        == DocumentRef {
                            id: uuid_v7,
                            ver: uuid_v7,
                        })
                    && (doc.doc_meta().reply().unwrap()
                        == DocumentRef {
                            id: uuid_v7,
                            ver: uuid_v7,
                        })
                    && (doc.doc_meta().template().unwrap()
                        == DocumentRef {
                            id: uuid_v7,
                            ver: uuid_v7,
                        })
                    && (doc.doc_meta().parameters().unwrap()
                        == DocumentRef {
                            id: uuid_v7,
                            ver: uuid_v7,
                        })
                    && (doc.doc_meta().section().unwrap() == &"$".parse::<Section>().unwrap())
                    && (doc.doc_meta().collabs() == &["Alex1".to_string(), "Alex2".to_string()])
                    && (doc.doc_content().decoded_bytes().unwrap()
                        == serde_json::to_vec(&serde_json::Value::Null).unwrap())
                    && (doc.kids() == vec![kid.clone()])
            }
        })),
    }
}

#[test]
fn catalyst_signed_doc_decoding_test() {
    let test_cases = vec![
        decoding_empty_bytes_case(),
        signed_doc_with_all_fields_case(),
    ];

    for case in test_cases.iter() {
        let bytes = case.bytes_gen.as_ref()();
        let doc_res = CatalystSignedDocument::try_from(bytes.as_slice());
        assert_eq!(doc_res.is_ok(), case.can_decode, "Case: [{}]", case.name);
        if let Ok(doc) = doc_res {
            assert_eq!(
                !doc.problem_report().is_problematic(),
                case.valid_doc,
                "Case: [{}]. Problem report: {:?}",
                case.name,
                doc.problem_report()
            );

            if let Some(post_checks) = &case.post_checks {
                assert!((post_checks.as_ref())(&doc), "Case: [{}]", case.name);
            }

            assert_eq!(
                bytes,
                Vec::<u8>::try_from(doc).unwrap(),
                "Case: [{}]. Asymetric encoding and decoding procedure",
                case.name
            );
        }
    }
}
