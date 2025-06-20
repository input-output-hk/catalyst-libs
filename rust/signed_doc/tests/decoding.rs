//! Integration test for COSE decoding part.

use catalyst_signed_doc::{doc_types::deprecated, providers::tests::TestVerifyingKeyProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use common::create_dummy_key_pair;
use coset::{CborSerializable, TaggedCborSerializable};
use ed25519_dalek::ed25519::signature::Signer;
use minicbor::{data::Tag, Encoder};

mod common;

#[tokio::test]
async fn catalyst_signed_doc_parameters_aliases_test() {
    catalyst_signed_doc_parameters_aliases(common::test_metadata()).await;
    catalyst_signed_doc_parameters_aliases(common::test_metadata_specific_type(
        Some(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.try_into().unwrap()),
        None,
    ))
    .await;
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::too_many_lines)]
async fn catalyst_signed_doc_parameters_aliases(data: (UuidV7, UuidV4, serde_json::Value)) {
    let (_, _, metadata_fields) = data;
    let (sk, pk, kid) = common::create_dummy_key_pair(RoleId::Role0).unwrap();
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid.clone(), pk);

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .unwrap()
        .build();
    assert!(!doc.problem_report().is_problematic());

    let parameters_val = doc.doc_meta().parameters().unwrap();
    let parameters_val_cbor =
        coset::cbor::Value::from_slice(minicbor::to_vec(parameters_val).unwrap().as_slice())
            .unwrap();
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
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
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
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
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
        .add_signature(|m| sk.sign(&m).to_vec(), kid)
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

type PostCheck = dyn Fn(&CatalystSignedDocument) -> bool;

struct TestCase {
    name: &'static str,
    bytes_gen: Box<dyn Fn() -> anyhow::Result<Encoder<Vec<u8>>>>,
    // If the provided bytes can be even decoded without error (valid COSE or not).
    // If set to `false` all further checks will not even happen.
    can_decode: bool,
    // If the decoded doc is a valid `CatalystSignedDocument`, underlying problem report is empty.
    valid_doc: bool,
    post_checks: Option<Box<PostCheck>>,
}

fn decoding_empty_bytes_case() -> TestCase {
    TestCase {
        name: "Decoding empty bytes",
        bytes_gen: Box::new(|| Ok(Encoder::new(Vec::new()))),
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

#[allow(clippy::unwrap_used)]
fn signed_doc_with_all_fields_case() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, signed (one signature), CBOR tagged.",
        bytes_gen: Box::new({
            move || {
                let (_, _, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers.str("id")?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers.str("ver")?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                e.bytes(p_headers.into_writer().as_slice())?;
                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // one signature
                e.array(1)?;
                e.array(3)?;
                // protected headers (kid field)
                let mut p_headers = minicbor::Encoder::new(Vec::new());
                p_headers.map(1)?.u8(4)?.encode(kid)?;
                e.bytes(p_headers.into_writer().as_slice())?;
                e.map(0)?;
                e.bytes(&[1,2,3])?;
                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                (doc.doc_type().unwrap() == &DocType::from(uuid_v4))
                    && (doc.doc_id().unwrap() == uuid_v7)
                    && (doc.doc_ver().unwrap() == uuid_v7)
                    && (doc.doc_content_type().unwrap() == ContentType::Json)
                    && (doc.encoded_content()
                        == serde_json::to_vec(&serde_json::Value::Null).unwrap()) && doc.kids().len() == 1
            }
        })),
    }
}

#[test]
fn catalyst_signed_doc_decoding_test() {
    let test_cases = [
        decoding_empty_bytes_case(),
        signed_doc_with_all_fields_case(),
    ];

    for case in test_cases {
        let bytes = case.bytes_gen.as_ref()().unwrap().into_writer();
        let doc_res = CatalystSignedDocument::try_from(bytes.as_slice());
        assert_eq!(
            doc_res.is_ok(),
            case.can_decode,
            "Case: [{}], error: {:?}",
            case.name,
            doc_res.err()
        );
        if let Ok(doc) = doc_res {
            assert_eq!(
                !doc.problem_report().is_problematic(),
                case.valid_doc,
                "Case: [{}]. Problem report: {:?}",
                case.name,
                doc.problem_report()
            );

            if let Some(post_checks) = &case.post_checks {
                assert!(
                    (post_checks.as_ref())(&doc),
                    "Case: [{}]. Post checks fails",
                    case.name
                );
            }

            assert_eq!(
                bytes,
                Vec::<u8>::try_from(doc).unwrap(),
                "Case: [{}]. Asymmetric encoding and decoding procedure",
                case.name
            );
        }
    }
}
