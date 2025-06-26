//! Integration test for signature validation part.

use std::io::Write;

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
        .unwrap()
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

fn content(
    content_bytes: &[u8], sk: &ed25519_dalek::SigningKey, kid: &CatalystId,
) -> anyhow::Result<minicbor::Encoder<Vec<u8>>> {
    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(4)?;
    // protected headers (empty metadata fields)
    let mut m_p_headers = minicbor::Encoder::new(Vec::new());
    m_p_headers.map(0)?;
    let m_p_headers = m_p_headers.into_writer();
    e.bytes(m_p_headers.as_slice())?;
    // empty unprotected headers
    e.map(0)?;
    // content
    e.writer_mut().write(content_bytes)?;
    // signatures
    // one signature
    e.array(1)?;
    e.array(3)?;
    // protected headers (kid field)
    let mut s_p_headers = minicbor::Encoder::new(Vec::new());
    s_p_headers
        .map(1)?
        .u8(4)?
        .bytes(Vec::<u8>::from(kid).as_slice())?;
    let s_p_headers = s_p_headers.into_writer();

    // [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4)
    let mut tbs: minicbor::Encoder<Vec<u8>> = minicbor::Encoder::new(Vec::new());
    tbs.array(5)?;
    tbs.str("Signature")?;
    tbs.bytes(&m_p_headers)?; // `body_protected`
    tbs.bytes(&s_p_headers)?; // `sign_protected`
    tbs.bytes(&[])?; // empty `external_aad`
    tbs.writer_mut().write_all(content_bytes)?; // `payload`

    e.bytes(s_p_headers.as_slice())?;
    e.map(0)?;
    e.bytes(&sk.sign(tbs.writer()).to_bytes())?;
    Ok(e)
}

fn parameters_aliase_field(
    alias: &str, sk: &ed25519_dalek::SigningKey, kid: &CatalystId,
) -> anyhow::Result<minicbor::Encoder<Vec<u8>>> {
    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(4)?;
    // protected headers (empty metadata fields)
    let mut m_p_headers = minicbor::Encoder::new(Vec::new());
    m_p_headers.map(0)?;
    let m_p_headers = m_p_headers.into_writer();
    e.bytes(m_p_headers.as_slice())?;
    // empty unprotected headers
    e.map(1)?;
    e.str(alias)?.encode_with(
        DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default()),
        &mut (),
    )?;
    // content (random bytes)
    let content = [1, 2, 3];
    e.bytes(&content)?;
    // signatures
    // one signature
    e.array(1)?;
    e.array(3)?;
    // protected headers (kid field)
    let mut s_p_headers = minicbor::Encoder::new(Vec::new());
    s_p_headers
        .map(1)?
        .u8(4)?
        .bytes(Vec::<u8>::from(kid).as_slice())?;
    let s_p_headers = s_p_headers.into_writer();

    // [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4)
    let mut tbs: minicbor::Encoder<Vec<u8>> = minicbor::Encoder::new(Vec::new());
    tbs.array(5)?;
    tbs.str("Signature")?;
    tbs.bytes(&m_p_headers)?; // `body_protected`
    tbs.bytes(&s_p_headers)?; // `sign_protected`
    tbs.bytes(&[])?; // empty `external_aad`
    tbs.bytes(&content)?; // `payload`

    e.bytes(s_p_headers.as_slice())?;
    e.map(0)?;
    e.bytes(&sk.sign(tbs.writer()).to_bytes())?;
    Ok(e)
}

#[tokio::test]
async fn special_cbor_cases() {
    let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
    let mut provider = TestVerifyingKeyProvider::default();
    provider.add_pk(kid.clone(), pk);

    struct SpecialCborTestCase<'a> {
        name: &'static str,
        doc_bytes_fn: &'a dyn Fn(
            &ed25519_dalek::SigningKey,
            &CatalystId,
        ) -> anyhow::Result<minicbor::Encoder<Vec<u8>>>,
    }

    let test_cases: &[SpecialCborTestCase] = &[
        SpecialCborTestCase {
            name: "content encoded as cbor null",
            doc_bytes_fn: &|sk, kid| {
                let mut e = minicbor::Encoder::new(Vec::new());
                content(e.null()?.writer().as_slice(), sk, kid)
            },
        },
        SpecialCborTestCase {
            name: "content encoded empty bstr e.g. &[]",
            doc_bytes_fn: &|sk, kid| {
                let mut e = minicbor::Encoder::new(Vec::new());
                content(e.bytes(&[])?.writer().as_slice(), sk, kid)
            },
        },
        SpecialCborTestCase {
            name: "parameters alias `category_id` field",
            doc_bytes_fn: &|sk, kid| parameters_aliase_field("category_id", sk, kid),
        },
        SpecialCborTestCase {
            name: "parameters alias `brand_id` field",
            doc_bytes_fn: &|sk, kid| parameters_aliase_field("brand_id", sk, kid),
        },
        SpecialCborTestCase {
            name: "`parameters` alias `campaign_id` field",
            doc_bytes_fn: &|sk, kid| parameters_aliase_field("campaign_id", sk, kid),
        },
    ];

    for case in test_cases {
        let doc = CatalystSignedDocument::try_from(
            (case.doc_bytes_fn)(&sk, &kid)
                .unwrap()
                .into_writer()
                .as_slice(),
        )
        .unwrap();

        assert!(
            validator::validate_signatures(&doc, &provider)
                .await
                .unwrap(),
            "[case: {}] {:?}",
            case.name,
            doc.problem_report()
        );
    }
}
