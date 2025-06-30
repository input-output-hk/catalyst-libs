//! Integration test for COSE decoding part.

use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;
use common::create_dummy_key_pair;
use minicbor::{data::Tag, Encoder};
use rand::Rng;

mod common;

type PostCheck = dyn Fn(&CatalystSignedDocument) -> anyhow::Result<()>;

struct TestCase {
    name: String,
    bytes_gen: Box<dyn Fn() -> anyhow::Result<Encoder<Vec<u8>>>>,
    // If the provided bytes can be even decoded without error (valid COSE or not).
    // If set to `false` all further checks will not even happen.
    can_decode: bool,
    // If the decoded doc is a valid `CatalystSignedDocument`, underlying problem report is empty.
    valid_doc: bool,
    post_checks: Option<Box<PostCheck>>,
}

fn signed_doc_with_valid_alias_case(alias: &'static str) -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let doc_ref = DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default());
    let doc_ref_cloned = doc_ref.clone();

    TestCase {
        name: format!("Provided '{alias}' field should be processed as parameters."),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str(alias)?
                        .encode_with(doc_ref.clone(), &mut ())?;

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                let cmp = DocumentRefs::from(vec![doc_ref_cloned.clone()]);
                anyhow::ensure!(doc.doc_meta().parameters() == Some(&cmp));
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_missing_header_field_case(field: &'static str) -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let doc_ref = DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default());

    TestCase {
        name: format!("Catalyst Signed Doc with missing '{field}' header."),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(4)?;
                    if field != "content-type" {
                        p_headers.u8(3)?.encode(ContentType::Json)?;
                    }
                    if field != "type" {
                        p_headers
                            .str("type")?
                            .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }
                    if field != "id" {
                        p_headers
                            .str("id")?
                            .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }
                    if field != "ver" {
                        p_headers
                            .str("ver")?
                            .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }

                    p_headers
                        .str("parameters")?
                        .encode_with(doc_ref.clone(), &mut ())?;

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
                if field == "content-type" {
                    anyhow::ensure!(doc.doc_meta().content_type().is_err());
                }
                if field == "type" {
                    anyhow::ensure!(doc.doc_meta().doc_type().is_err());
                }
                if field == "id" {
                    anyhow::ensure!(doc.doc_meta().doc_id().is_err());
                }
                if field == "ver" {
                    anyhow::ensure!(doc.doc_meta().doc_ver().is_err());
                }

                Ok(())
            }
        })),
    }
}

fn signed_doc_with_random_header_field_case(field: &'static str) -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let doc_ref = DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default());

    TestCase {
        name: format!("Catalyst Signed Doc with random bytes in '{field}' header field."),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut rng = rand::thread_rng();
                    let mut rand_buf = [0u8; 128];
                    rng.try_fill(&mut rand_buf)?;

                    let is_required_header = ["type", "id", "ver", "parameters"]
                        .iter()
                        .any(|v| v == &field);

                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(if is_required_header { 5 } else { 6 })?;
                    if field == "content-type" {
                        p_headers.u8(3)?.encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers.u8(3)?.encode(ContentType::Json)?;
                    }
                    if field == "type" {
                        p_headers.str("type")?.encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers
                            .str("type")?
                            .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }
                    if field == "id" {
                        p_headers.str("id")?.encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers
                            .str("id")?
                            .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }
                    if field == "ver" {
                        p_headers.str("ver")?.encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers
                            .str("ver")?
                            .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    }
                    if field == "parameters" {
                        p_headers
                            .str("parameters")?
                            .encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers
                            .str("parameters")?
                            .encode_with(doc_ref.clone(), &mut ())?;
                    }

                    if !is_required_header {
                        p_headers.str(field)?.encode_with(rand_buf, &mut ())?;
                    }

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_meta().content_encoding().is_none());
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().section().is_none());
                anyhow::ensure!(doc.doc_meta().collabs().is_empty());

                if field == "content-type" {
                    anyhow::ensure!(doc.doc_meta().content_type().is_err());
                }
                if field == "type" {
                    anyhow::ensure!(doc.doc_meta().doc_type().is_err());
                }
                if field == "id" {
                    anyhow::ensure!(doc.doc_meta().doc_id().is_err());
                }
                if field == "ver" {
                    anyhow::ensure!(doc.doc_meta().doc_ver().is_err());
                }
                if field == "parameters" {
                    anyhow::ensure!(doc.doc_meta().parameters().is_none());
                }

                Ok(())
            }
        })),
    }
}

// `parameters` value along with its aliases are not allowed to be presented
fn signed_doc_with_parameters_and_aliases_case(aliases: &'static [&'static str]) -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    let doc_ref = DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default());

    TestCase {
        name: format!("Multiple definitions of '{}' at once.", aliases.join(", ")),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(4u64.overflowing_add(u64::try_from(aliases.len())?).0)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    for alias in aliases {
                        p_headers
                            .str(alias)?
                            .encode_with(doc_ref.clone(), &mut ())?;
                    }

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_content_encoding_case(upper: bool) -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    let name = if upper {
        "Content-Encoding"
    } else {
        "content-encoding"
    };

    TestCase {
        name: format!("content_encoding field, allow upper and lower case key value: '{name}'"),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers.str(name)?.encode(ContentEncoding::Brotli)?;

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(matches!(
                    doc.doc_meta().content_encoding(),
                    Some(ContentEncoding::Brotli)
                ));
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_random_kid_case() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    TestCase {
        name: "Invalid signature kid field format (random bytes)".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // one signature
                e.array(1)?;
                e.array(3)?;
                // protected headers (kid field)
                e.bytes({
                    let mut rng = rand::thread_rng();
                    let mut rand_buf = [0u8; 128];
                    rng.try_fill(&mut rand_buf)?;

                    let mut p_headers = minicbor::Encoder::new(Vec::new());
                    p_headers.map(1)?.u8(4)?.bytes(&rand_buf)?;

                    p_headers.into_writer().as_slice()
                })?;
                e.map(0)?;
                e.bytes(&[1, 2, 3])?;

                Ok(e)
            }
        }),
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_wrong_cose_sign_tag_case() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    TestCase {
        name: "Catalyst Signed Doc with wrong COSE sign tag value (not `98`)".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(u64::MAX))?;
                e.array(4)?;

                // protected headers (metadata fields)
                e.bytes({
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    p_headers.into_writer().as_slice()
                })?;

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // no signature
                e.array(0)?;

                Ok(e)
            }
        }),
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn decoding_empty_bytes_case() -> TestCase {
    TestCase {
        name: "Decoding empty bytes".to_string(),
        bytes_gen: Box::new(|| Ok(Encoder::new(Vec::new()))),
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_all_fields_case() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, signed (one signature), CBOR tagged.".to_string(),
        bytes_gen: Box::new({
            move || {
                let (_, _, kid) = create_dummy_key_pair(RoleId::Role0)?;

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
                p_headers.map(1)?.u8(4)?.bytes(Vec::<u8>::from(&kid).as_slice())?;
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
                anyhow::ensure!(doc.doc_type()? == &DocType::from(uuid_v4));
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type()? == ContentType::Json);
                anyhow::ensure!(doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?);
                anyhow::ensure!(doc.kids().len() == 1);
                Ok(())
            }
        })),
    }
}

#[allow(clippy::unwrap_used)]
fn minimally_valid_tagged_signed_doc() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, unsigned, CBOR tagged."
            .to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers
                    .str("type")?
                    .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("id")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("ver")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                e.bytes(p_headers.into_writer().as_slice())?;
                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // no signature
                e.array(0)?;
                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type().unwrap() == &DocType::from(uuid_v4));
                anyhow::ensure!(doc.doc_id().unwrap() == uuid_v7);
                anyhow::ensure!(doc.doc_ver().unwrap() == uuid_v7);
                anyhow::ensure!(doc.doc_content_type().unwrap() == ContentType::Json);
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().parameters().is_none());
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null).unwrap()
                );
                Ok(())
            }
        })),
    }
}

#[allow(clippy::unwrap_used)]
fn minimally_valid_untagged_signed_doc() -> TestCase {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, unsigned, CBOR tagged."
            .to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers
                    .str("type")?
                    .encode_with(uuid_v4, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("id")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("ver")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                e.bytes(p_headers.into_writer().as_slice())?;
                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // no signature
                e.array(0)?;
                Ok(e)
            }
        }),
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type().unwrap() == &DocType::from(uuid_v4));
                anyhow::ensure!(doc.doc_id().unwrap() == uuid_v7);
                anyhow::ensure!(doc.doc_ver().unwrap() == uuid_v7);
                anyhow::ensure!(doc.doc_content_type().unwrap() == ContentType::Json);
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().parameters().is_none());
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null).unwrap()
                );
                Ok(())
            }
        })),
    }
}

#[test]
fn catalyst_signed_doc_decoding_test() {
    let test_cases = [
        decoding_empty_bytes_case(),
        signed_doc_with_all_fields_case(),
        signed_doc_with_random_kid_case(),
        signed_doc_with_wrong_cose_sign_tag_case(),
        signed_doc_with_content_encoding_case(true),
        signed_doc_with_content_encoding_case(false),
        signed_doc_with_valid_alias_case("category_id"),
        signed_doc_with_valid_alias_case("brand_id"),
        signed_doc_with_valid_alias_case("campaign_id"),
        signed_doc_with_missing_header_field_case("content-type"),
        signed_doc_with_missing_header_field_case("type"),
        signed_doc_with_missing_header_field_case("id"),
        signed_doc_with_missing_header_field_case("ver"),
        signed_doc_with_random_header_field_case("content-type"),
        signed_doc_with_random_header_field_case("type"),
        signed_doc_with_random_header_field_case("id"),
        signed_doc_with_random_header_field_case("ver"),
        signed_doc_with_random_header_field_case("ref"),
        signed_doc_with_random_header_field_case("template"),
        signed_doc_with_random_header_field_case("reply"),
        signed_doc_with_random_header_field_case("section"),
        signed_doc_with_random_header_field_case("collabs"),
        signed_doc_with_random_header_field_case("parameters"),
        signed_doc_with_random_header_field_case("content-encoding"),
        signed_doc_with_parameters_and_aliases_case(&["parameters", "category_id"]),
        signed_doc_with_parameters_and_aliases_case(&["parameters", "brand_id"]),
        signed_doc_with_parameters_and_aliases_case(&["parameters", "campaign_id"]),
        signed_doc_with_parameters_and_aliases_case(&["category_id", "campaign_id"]),
        signed_doc_with_parameters_and_aliases_case(&["category_id", "brand_id"]),
        signed_doc_with_parameters_and_aliases_case(&["brand_id", "campaign_id"]),
        signed_doc_with_parameters_and_aliases_case(&["category_id", "brand_id", "campaign_id"]),
        signed_doc_with_parameters_and_aliases_case(&[
            "parameters",
            "category_id",
            "brand_id",
            "campaign_id",
        ]),
        minimally_valid_tagged_signed_doc(),
        minimally_valid_untagged_signed_doc(),
    ];

    for case in test_cases {
        let bytes_res = case.bytes_gen.as_ref()();
        assert!(
            bytes_res.is_ok(),
            "Case: [{}], error: {:?}",
            case.name,
            bytes_res.err()
        );
        let bytes = bytes_res.unwrap().into_writer();
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
                let post_checks_res = post_checks(&doc);
                assert!(
                    post_checks_res.is_ok(),
                    "Case: [{}]. Post checks fails: {:?}",
                    case.name,
                    post_checks_res.err()
                );
            }
        }
    }
}
