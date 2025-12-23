use std::io::Write;

use catalyst_types::{
    catalyst_id::{CatalystId, role_index::RoleId},
    uuid::{UuidV4, UuidV7},
};
use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::{
    providers::tests::*,
    tests_utils::{create_dummy_doc_ref, create_dummy_key_pair},
    *,
};

fn metadata() -> serde_json::Value {
    let ref_doc = create_dummy_doc_ref();
    let reply_doc = create_dummy_doc_ref();
    let template_doc = create_dummy_doc_ref();
    let parameters_doc = create_dummy_doc_ref();

    serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": UuidV4::new(),
        "id":  UuidV7::new(),
        "ver":  UuidV7::new(),
        "ref": [ref_doc],
        "reply": [reply_doc],
        "template": [template_doc],
        "section": "$",
        "collaborators": vec![
            /* cspell:disable */
            "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
            "id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3"
            /* cspell:enable */
        ],
        "parameters": [parameters_doc],
    })
}

#[test]
fn single_signature_validation_test() {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);

    let signed_doc = Builder::new()
        .with_json_metadata(metadata())
        .unwrap()
        .with_json_content(&serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    assert!(!signed_doc.report().is_problematic());

    // case: has key
    let mut provider = TestCatalystProvider::default();
    provider.add_sk(kid.clone(), sk);
    assert!(
        SignatureRule::check_inner(&signed_doc, &provider).unwrap(),
        "{:?}",
        signed_doc.report()
    );

    // case: empty provider
    assert!(!SignatureRule::check_inner(&signed_doc, &TestCatalystProvider::default()).unwrap());

    // case: signed with different key
    let (another_sk, ..) = create_dummy_key_pair(RoleId::Role0);
    let invalid_doc = signed_doc
        .into_builder()
        .unwrap()
        .add_signature(|m| another_sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();
    assert!(!SignatureRule::check_inner(&invalid_doc, &provider).unwrap());

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
    assert!(!SignatureRule::check_inner(&unsigned_doc, &provider).unwrap());
}

#[test]
fn multiple_signatures_validation_test() {
    let (sk1, kid1) = create_dummy_key_pair(RoleId::Role0);
    let (sk2, kid2) = create_dummy_key_pair(RoleId::Role0);
    let (sk3, kid3) = create_dummy_key_pair(RoleId::Role0);
    let (sk_n, kid_n) = create_dummy_key_pair(RoleId::Role0);

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

    assert!(!signed_doc.report().is_problematic());

    // case: all signatures valid
    let mut provider = TestCatalystProvider::default();
    provider.add_sk(kid1.clone(), sk1.clone());
    provider.add_sk(kid2.clone(), sk2.clone());
    provider.add_sk(kid3.clone(), sk3.clone());
    assert!(SignatureRule::check_inner(&signed_doc, &provider).unwrap());

    // case: partially available signatures
    let mut provider = TestCatalystProvider::default();
    provider.add_sk(kid1.clone(), sk1);
    provider.add_sk(kid2.clone(), sk2);
    assert!(!SignatureRule::check_inner(&signed_doc, &provider).unwrap());

    // case: with unrecognized provider
    let mut provider = TestCatalystProvider::default();
    provider.add_sk(kid_n.clone(), sk_n);
    assert!(!SignatureRule::check_inner(&signed_doc, &provider).unwrap());

    // case: no valid signatures available
    assert!(!SignatureRule::check_inner(&signed_doc, &TestCatalystProvider::default()).unwrap());
}

fn content(
    content_bytes: &[u8],
    sk: &ed25519_dalek::SigningKey,
    kid: &CatalystId,
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
    let _ = e.writer_mut().write(content_bytes)?;
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

fn parameters_alias_field(
    alias: &str,
    sk: &ed25519_dalek::SigningKey,
    kid: &CatalystId,
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
    e.str(alias)?.encode_with(create_dummy_doc_ref(), &mut ())?;
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

type DocBytesGenerator =
    dyn Fn(&ed25519_dalek::SigningKey, &CatalystId) -> anyhow::Result<minicbor::Encoder<Vec<u8>>>;

struct SpecialCborTestCase<'a> {
    name: &'static str,
    doc_bytes_fn: &'a DocBytesGenerator,
}

#[test]
fn special_cbor_cases() {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
    let mut provider = TestCatalystProvider::default();
    provider.add_sk(kid.clone(), sk.clone());

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
            doc_bytes_fn: &|sk, kid| parameters_alias_field("category_id", sk, kid),
        },
        SpecialCborTestCase {
            name: "parameters alias `brand_id` field",
            doc_bytes_fn: &|sk, kid| parameters_alias_field("brand_id", sk, kid),
        },
        SpecialCborTestCase {
            name: "`parameters` alias `campaign_id` field",
            doc_bytes_fn: &|sk, kid| parameters_alias_field("campaign_id", sk, kid),
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
            SignatureRule::check_inner(&doc, &provider).unwrap(),
            "[case: {}] {:?}",
            case.name,
            doc.report()
        );
    }
}
