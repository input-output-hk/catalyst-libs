use super::*;
use crate::{builder::tests::Builder, metadata::SupportedField};

#[test]
fn cbor_with_trailing_bytes_test() {
    // valid cbor: {1: 2} but with trailing 0xff
    let mut buf = Vec::new();
    let mut enc = minicbor::Encoder::new(&mut buf);
    enc.map(1).unwrap().u8(1).unwrap().u8(2).unwrap();
    buf.push(0xFF); // extra byte

    let content_type = ContentType::Cbor;
    let cbor_rule = ContentTypeRule::Specified { exp: content_type };

    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(buf)
        .build();

    assert!(matches!(cbor_rule.check_inner(&doc), Ok(false)));
}

#[test]
fn malformed_cbor_bytes_test() {
    // 0xa2 means a map with 2 key-value pairs, but we only give 1 key
    let invalid_bytes = &[0xA2, 0x01];

    let content_type = ContentType::Cbor;
    let cbor_rule = ContentTypeRule::Specified { exp: content_type };

    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(invalid_bytes.into())
        .build();

    assert!(matches!(cbor_rule.check_inner(&doc), Ok(false)));
}

#[test]
fn content_type_cbor_rule_test() {
    let content_type = ContentType::Cbor;
    let cbor_rule = ContentTypeRule::Specified { exp: content_type };

    // with json bytes
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
        .build();
    assert!(matches!(cbor_rule.check_inner(&doc), Ok(false)));

    // with cbor bytes
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
        .build();
    assert!(matches!(cbor_rule.check_inner(&doc), Ok(true)));

    // without content
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .build();
    assert!(matches!(cbor_rule.check_inner(&doc), Ok(false)));

    // with empty content
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .build();
    assert!(matches!(cbor_rule.check_inner(&doc), Ok(false)));
}

#[test]
fn content_type_json_rule_test() {
    let content_type = ContentType::Json;
    let json_rule = ContentTypeRule::Specified {
        exp: ContentType::Json,
    };

    // with json bytes
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(serde_json::to_vec(&serde_json::json!({})).unwrap())
        .build();
    assert!(matches!(json_rule.check_inner(&doc), Ok(true)));

    // with cbor bytes
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .with_content(minicbor::to_vec(minicbor::data::Token::Null).unwrap())
        .build();
    assert!(matches!(json_rule.check_inner(&doc), Ok(false)));

    // without content
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .build();
    assert!(matches!(json_rule.check_inner(&doc), Ok(false)));

    // with empty content
    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .build();
    assert!(matches!(json_rule.check_inner(&doc), Ok(false)));

    let doc = Builder::new().build();
    assert!(matches!(json_rule.check_inner(&doc), Ok(false)));
}

#[test]
fn content_type_not_specified_rule_test() {
    let content_type = ContentType::Json;
    let rule = ContentTypeRule::NotSpecified;

    let doc = Builder::new()
        .with_metadata_field(SupportedField::ContentType(content_type))
        .build();
    assert!(!rule.check_inner(&doc).unwrap());

    let doc = Builder::new().build();
    assert!(rule.check_inner(&doc).unwrap());
}
