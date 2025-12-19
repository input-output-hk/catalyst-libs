//! Integration test for COSE decoding part.

use catalyst_signed_doc::{
    decode_context::CompatibilityPolicy,
    tests_utils::{create_dummy_doc_ref, create_dummy_key_pair},
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use minicbor::{Decode, Encoder, data::Tag};
use rand::Rng;

type PostCheck = dyn Fn(&CatalystSignedDocument) -> anyhow::Result<()>;

struct TestCase {
    name: String,
    bytes_gen: Box<dyn Fn() -> anyhow::Result<Encoder<Vec<u8>>>>,
    policy: CompatibilityPolicy,
    // If the provided bytes can be even decoded without error (valid COSE or not).
    // If set to `false` all further checks will not even happen.
    can_decode: bool,
    // If the decoded doc is a valid `CatalystSignedDocument`, underlying problem report is empty.
    valid_doc: bool,
    post_checks: Option<Box<PostCheck>>,
}

#[allow(clippy::unwrap_used)]
fn signed_doc_deprecated_doc_ref_case(field_name: &'static str) -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    let doc_ref = create_dummy_doc_ref();
    TestCase {
        name: format!(
            "Catalyst Signed Doc with deprecated {field_name} version before v0.04 validating."
        ),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers.str(field_name)?;
                    p_headers.array(2)?;
                    p_headers.encode_with(
                        doc_ref.id(),
                        &mut catalyst_types::uuid::CborContext::Tagged,
                    )?;
                    p_headers.encode_with(
                        doc_ref.ver(),
                        &mut catalyst_types::uuid::CborContext::Tagged,
                    )?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

#[allow(clippy::unwrap_used)]
fn signed_doc_with_valid_alias_case(alias: &'static str) -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    let doc_ref = DocumentRefs::from(vec![create_dummy_doc_ref()]);
    let doc_ref_cloned = doc_ref.clone();
    TestCase {
        name: format!("Provided '{alias}' field should be processed as parameters."),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str(alias)?
                        .encode_with(doc_ref.clone(), &mut ())?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                let cmp = doc_ref_cloned.clone();
                anyhow::ensure!(doc.doc_meta().parameters() == Some(&cmp));
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_missing_header_field_case(field: &'static str) -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: format!("Catalyst Signed Doc with missing '{field}' header."),
        bytes_gen: Box::new({
            move || {
                let doc_ref = create_dummy_doc_ref();
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(3)?;
                    if field != "type" {
                        p_headers.str("type")?.encode(&doc_type)?;
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

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
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
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: format!("Catalyst Signed Doc with random bytes in '{field}' header field."),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut rng = rand::thread_rng();
                    let mut rand_buf = [0u8; 128];
                    rng.try_fill(&mut rand_buf)?;

                    let is_required_header = ["type", "id", "ver"].iter().any(|v| v == &field);

                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(if is_required_header { 3 } else { 4 })?;
                    if field == "type" {
                        p_headers.str("type")?.encode_with(rand_buf, &mut ())?;
                    } else {
                        p_headers.str("type")?.encode(&doc_type)?;
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

                    if !is_required_header {
                        p_headers.str(field)?.encode_with(rand_buf, &mut ())?;
                    }

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_meta().content_type().is_none());
                anyhow::ensure!(doc.doc_meta().content_encoding().is_none());
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().section().is_none());
                anyhow::ensure!(doc.doc_meta().parameters().is_none());
                anyhow::ensure!(doc.doc_meta().collaborators().is_empty());

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

// `parameters` value along with its aliases are not allowed to be presented
fn signed_doc_with_parameters_and_aliases_case(aliases: &'static [&'static str]) -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: format!("Multiple definitions of '{}' at once.", aliases.join(", ")),
        bytes_gen: Box::new({
            move || {
                let doc_ref = create_dummy_doc_ref();
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(4u64.overflowing_add(u64::try_from(aliases.len())?).0)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
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

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_content_encoding_case(upper: bool) -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
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
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers.str(name)?.encode(ContentEncoding::Brotli)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // zero signatures
                e.array(0)?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
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
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Invalid signature kid field format (random bytes)".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // one signature
                e.array(1)?;
                e.array(3)?;
                // protected headers (kid field)
                {
                    let mut rng = rand::thread_rng();
                    let mut rand_buf = [0u8; 128];
                    rng.try_fill(&mut rand_buf)?;

                    let mut p_headers = minicbor::Encoder::new(Vec::new());
                    p_headers.map(1)?.u8(4)?.bytes(&rand_buf)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
                e.map(0)?;
                e.bytes(&[1, 2, 3])?;

                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_wrong_cose_tag_case() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with wrong COSE sign tag value (not `98`)".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(u64::MAX))?;
                e.array(4)?;

                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());
                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }

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
        policy: CompatibilityPolicy::Accept,
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn decoding_empty_bytes_case() -> TestCase {
    TestCase {
        name: "Decoding empty bytes".to_string(),
        bytes_gen: Box::new(|| Ok(Encoder::new(Vec::new()))),
        policy: CompatibilityPolicy::Accept,
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_minimal_metadata_fields_case() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, signed (one signature), CBOR tagged.".to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(4)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
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
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                anyhow::ensure!(doc.authors().len() == 1);
                anyhow::ensure!(!doc.is_deprecated()?);
                Ok(())
            }
        })),
    }
}

#[allow(clippy::unwrap_used)]
fn signed_doc_with_complete_metadata_fields_case() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    let doc_ref = DocumentRefs::from(vec![create_dummy_doc_ref()]);
    let doc_ref_cloned = doc_ref.clone();
    TestCase {
        name: "Catalyst Signed Doc with all metadata fields defined, signed (one signature), CBOR tagged.".to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            let doc_ref = doc_ref.clone();
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(9)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
                p_headers.str("id")?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers.str("ver")?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("ref")?
                    .encode_with(doc_ref.clone(), &mut ())?;
                p_headers
                    .str("template")?
                    .encode_with(doc_ref.clone(), &mut ())?;
                p_headers
                    .str("reply")?
                    .encode_with(doc_ref.clone(), &mut ())?;
                p_headers.str("section")?.encode("$")?;

                p_headers.str("revocations")?;
                p_headers.array(1)?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                /* cspell:disable */
                p_headers.str("collaborators")?;
                p_headers.array(2)?;
                p_headers.bytes(b"id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE")?;
                p_headers.bytes(b"id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3")?;
                /* cspell:enable */
                p_headers.str("parameters")?.encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers.str("chain")?;
                p_headers.array(2)?;
                p_headers.int(0.into())?;
                p_headers.encode_with(doc_ref.clone(), &mut ())?;

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
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                let refs = doc_ref_cloned.clone();
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_meta().doc_ref() == Some(&refs));
                anyhow::ensure!(doc.doc_meta().template() == Some(&refs));
                anyhow::ensure!(doc.doc_meta().reply() == Some(&refs));
                anyhow::ensure!(doc.doc_meta().revocations() == Some(&vec![uuid_v7].into()));
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?);
                anyhow::ensure!(doc.authors().len() == 1);
                anyhow::ensure!(!doc.is_deprecated()?);
                Ok(())
            }
        })),
    }
}

fn minimally_valid_tagged_signed_doc() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, unsigned, CBOR tagged."
            .to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
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
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().parameters().is_none());
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                Ok(())
            }
        })),
    }
}

fn minimally_valid_untagged_signed_doc() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, unsigned, CBOR tagged."
            .to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let mut e = Encoder::new(Vec::new());
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
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
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(doc.doc_meta().doc_ref().is_none());
                anyhow::ensure!(doc.doc_meta().template().is_none());
                anyhow::ensure!(doc.doc_meta().reply().is_none());
                anyhow::ensure!(doc.doc_meta().parameters().is_none());
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                Ok(())
            }
        })),
    }
}

fn signed_doc_valid_null_as_no_content() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with 'content' defined as Null.".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
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
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                Ok(())
            }
        })),
    }
}

fn signed_doc_valid_empty_bstr_as_no_content() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with 'content' defined as empty bstr.".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
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
                e.bytes(&[])?;
                // signatures
                // no signature
                e.array(0)?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.encoded_content() == Vec::<u8>::new());
                Ok(())
            }
        })),
    }
}

fn signed_doc_valid_nil_content() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with CBOR nil 'content'.".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
                p_headers
                    .str("id")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("ver")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                e.bytes(p_headers.into_writer().as_slice())?;
                // empty unprotected headers
                e.map(0)?;
                // nil content
                e.null()?;
                // signatures
                // no signature
                e.array(0)?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.encoded_content() == Vec::<u8>::new());
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_non_empty_unprotected_headers() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with non empty unprotected headers".to_string(),
        bytes_gen: Box::new({
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
                p_headers
                    .str("id")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                p_headers
                    .str("ver")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                e.bytes(p_headers.into_writer().as_slice())?;
                // non empty unprotected headers
                e.map(1)?;
                e.str("id")?
                    .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                // no signature
                e.array(0)?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_signatures_non_empty_unprotected_headers() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with signatures non empty unprotected headers".to_string(),
        bytes_gen: Box::new({
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                let mut p_headers = Encoder::new(Vec::new());

                p_headers.map(4)?;
                p_headers.u8(3)?.encode(ContentType::Json)?;
                p_headers.str("type")?.encode(&doc_type)?;
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
                // one signature
                e.array(1)?;
                e.array(3)?;
                // protected headers (kid field)
                let mut p_headers = minicbor::Encoder::new(Vec::new());
                p_headers
                    .map(1)?
                    .u8(4)?
                    .bytes(Vec::<u8>::from(&kid).as_slice())?;
                e.bytes(p_headers.into_writer().as_slice())?;
                // non empty unprotected headers
                e.map(1)?.u8(4)?.bytes(Vec::<u8>::from(&kid).as_slice())?;
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_strict_deterministic_decoding_wrong_order() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, with enabled strictly decoded rules, metadata field in the wrong order".to_string(),
        bytes_gen: Box::new({
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(4)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
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
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Fail,
        can_decode: false,
        valid_doc: false,
        post_checks: None,
    }
}

fn signed_doc_with_non_strict_deterministic_decoding_wrong_order() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with minimally defined metadata fields, with enabled non strictly (warn) decoded rules, metadata field in the wrong order".to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(4)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers
                        .str("type")?
                        .encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
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
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Warn,
        can_decode: true,
        valid_doc: true,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                anyhow::ensure!(doc.authors().len() == 1);
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_non_supported_metadata_invalid() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with non-supported defined metadata fields is invalid."
            .to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(5)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("unsupported")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
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
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                anyhow::ensure!(doc.authors().len() == 0);
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_kid_in_id_form_invalid() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with Signature KID in Id form, instead of URI form is invalid."
            .to_string(),
        bytes_gen: Box::new({
            let doc_type = doc_type.clone();
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(4)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
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
                p_headers
                    .map(1)?
                    .u8(4)?
                    .bytes(Vec::<u8>::from(&kid.as_id()).as_slice())?;
                e.bytes(p_headers.into_writer().as_slice())?;
                e.map(0)?;
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: Some(Box::new({
            move |doc| {
                anyhow::ensure!(doc.doc_type()? == &doc_type);
                anyhow::ensure!(doc.doc_id()? == uuid_v7);
                anyhow::ensure!(doc.doc_ver()? == uuid_v7);
                anyhow::ensure!(doc.doc_content_type() == Some(ContentType::Json));
                anyhow::ensure!(
                    doc.encoded_content() == serde_json::to_vec(&serde_json::Value::Null)?
                );
                anyhow::ensure!(doc.authors().len() == 1);
                Ok(())
            }
        })),
    }
}

fn signed_doc_with_non_supported_protected_signature_header_invalid() -> TestCase {
    let uuid_v7 = uuid::UuidV7::new();
    let doc_type = DocType::from(uuid::UuidV4::new());
    TestCase {
        name: "Catalyst Signed Doc with unsupported protected Signature header is invalid."
            .to_string(),
        bytes_gen: Box::new({
            move || {
                let (_, kid) = create_dummy_key_pair(RoleId::Role0);

                let mut e = Encoder::new(Vec::new());
                e.tag(Tag::new(98))?;
                e.array(4)?;
                // protected headers (metadata fields)
                {
                    let mut p_headers = Encoder::new(Vec::new());

                    p_headers.map(4)?;
                    p_headers.u8(3)?.encode(ContentType::Json)?;
                    p_headers.str("type")?.encode(&doc_type)?;
                    p_headers
                        .str("id")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    p_headers
                        .str("ver")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;

                    e.bytes(p_headers.into_writer().as_slice())?;
                }
                // empty unprotected headers
                e.map(0)?;
                // content
                e.bytes(serde_json::to_vec(&serde_json::Value::Null)?.as_slice())?;
                // signatures
                e.array(1)?;
                // signature
                e.array(3)?;
                // protected headers
                {
                    let mut s_headers = minicbor::Encoder::new(Vec::new());
                    s_headers.map(2)?;
                    // (kid field)
                    s_headers.u8(4)?.bytes(Vec::<u8>::from(&kid).as_slice())?;
                    // Unsupported label/value
                    s_headers
                        .str("unsupported")?
                        .encode_with(uuid_v7, &mut catalyst_types::uuid::CborContext::Tagged)?;
                    e.bytes(s_headers.into_writer().as_slice())?;
                }
                // unprotected headers
                e.map(0)?;
                // signature bytes
                e.bytes(&[1, 2, 3])?;
                Ok(e)
            }
        }),
        policy: CompatibilityPolicy::Accept,
        can_decode: true,
        valid_doc: false,
        post_checks: None,
    }
}

#[test]
fn catalyst_signed_doc_decoding_test() {
    let test_cases = [
        decoding_empty_bytes_case(),
        signed_doc_deprecated_doc_ref_case("template"),
        signed_doc_deprecated_doc_ref_case("ref"),
        signed_doc_deprecated_doc_ref_case("reply"),
        signed_doc_deprecated_doc_ref_case("parameters"),
        signed_doc_deprecated_doc_ref_case("category_id"),
        signed_doc_deprecated_doc_ref_case("brand_id"),
        signed_doc_deprecated_doc_ref_case("campaign_id"),
        signed_doc_with_minimal_metadata_fields_case(),
        signed_doc_with_complete_metadata_fields_case(),
        signed_doc_valid_null_as_no_content(),
        signed_doc_valid_empty_bstr_as_no_content(),
        signed_doc_valid_nil_content(),
        signed_doc_with_random_kid_case(),
        signed_doc_with_wrong_cose_tag_case(),
        signed_doc_with_content_encoding_case(true),
        signed_doc_with_content_encoding_case(false),
        signed_doc_with_valid_alias_case("category_id"),
        signed_doc_with_valid_alias_case("brand_id"),
        signed_doc_with_valid_alias_case("campaign_id"),
        signed_doc_with_valid_alias_case("parameters"),
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
        signed_doc_with_random_header_field_case("revocations"),
        signed_doc_with_random_header_field_case("collaborators"),
        signed_doc_with_random_header_field_case("parameters"),
        signed_doc_with_random_header_field_case("chain"),
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
        signed_doc_with_non_empty_unprotected_headers(),
        signed_doc_with_signatures_non_empty_unprotected_headers(),
        signed_doc_with_strict_deterministic_decoding_wrong_order(),
        signed_doc_with_non_strict_deterministic_decoding_wrong_order(),
        signed_doc_with_non_supported_metadata_invalid(),
        signed_doc_with_kid_in_id_form_invalid(),
        signed_doc_with_non_supported_protected_signature_header_invalid(),
    ];

    for mut case in test_cases {
        let bytes_res = case.bytes_gen.as_ref()();
        assert!(
            bytes_res.is_ok(),
            "Case: [{}], error: {:?}",
            case.name,
            bytes_res.err()
        );
        let bytes = bytes_res.unwrap().into_writer();
        let doc_res =
            CatalystSignedDocument::decode(&mut minicbor::Decoder::new(&bytes), &mut case.policy);
        assert_eq!(
            doc_res.is_ok(),
            case.can_decode,
            "Case: [{}], error: {:?}",
            case.name,
            doc_res.err()
        );
        if let Ok(doc) = doc_res {
            assert_eq!(
                !doc.report().is_problematic(),
                case.valid_doc,
                "Case: [{}]. Problem report: {:?}",
                case.name,
                doc.report()
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
