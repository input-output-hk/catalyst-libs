//! Integration test for COSE decoding part.

use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;
use common::create_dummy_key_pair;
use coset::TaggedCborSerializable;
use ed25519_dalek::ed25519::signature::Signer;

mod common;

#[test]
fn catalyst_signed_doc_cbor_roundtrip_test() {
    let (uuid_v7, uuid_v4, metadata_fields) = common::test_metadata();
    let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();

    assert!(!doc.problem_report().is_problematic());

    let bytes: Vec<u8> = doc.try_into().unwrap();
    let decoded: CatalystSignedDocument = bytes.as_slice().try_into().unwrap();
    let extra_fields: ExtraFields = serde_json::from_value(metadata_fields).unwrap();

    assert_eq!(decoded.doc_type().unwrap(), uuid_v4);
    assert_eq!(decoded.doc_id().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_ver().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_content().decoded_bytes().unwrap(), &content);
    assert_eq!(decoded.doc_meta(), &extra_fields);
}

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

#[test]
fn catalyst_signed_doc_parameters_aliases_test() {
    let (_, _, metadata_fields) = common::test_metadata();

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .build();
    assert!(!doc.problem_report().is_problematic());

    let parameters_val = doc.doc_meta().parameters().unwrap();
    let parameters_val_cbor: coset::cbor::Value = parameters_val.try_into().unwrap();
    // replace parameters with the aliase values `category_id`, `brand_id`, `campaign_id`.
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

    let doc: CatalystSignedDocument = cose_with_category_id
        .to_tagged_vec()
        .unwrap()
        .as_slice()
        .try_into()
        .unwrap();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());

    // case: `brand_id`.
    let mut cose_with_category_id = cose.clone();
    cose_with_category_id.protected.header.rest.push((
        coset::Label::Text("brand_id".to_string()),
        parameters_val_cbor.clone(),
    ));

    let doc: CatalystSignedDocument = cose_with_category_id
        .to_tagged_vec()
        .unwrap()
        .as_slice()
        .try_into()
        .unwrap();
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());

    // case: `campaign_id`.
    let mut cose_with_category_id = cose.clone();
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
    assert!(!doc.problem_report().is_problematic());
    assert!(doc.doc_meta().parameters().is_some());

    // `parameters` value along with its alises are not allowed to be present at the
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
