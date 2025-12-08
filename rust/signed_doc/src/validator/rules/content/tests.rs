use catalyst_types::json_schema::JsonSchema;
use test_case::test_case;

use super::*;
use crate::builder::tests::Builder;

#[test_case(
    |valid_content| {
        Builder::new()
            .with_content(valid_content)
            .build()
    }
    => true
    ;
    "valid content"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_content(vec![1, 2, 3])
            .build()
    }
    => false
    ;
    "corrupted content"
)]
#[test_case(
    |_| {
        Builder::new()
            .build()
    }
    => false
    ;
    "missing content"
)]
fn content_rule_specified_test(doc_gen: impl FnOnce(Vec<u8>) -> CatalystSignedDocument) -> bool {
    let schema = JsonSchema::try_from(&serde_json::json!({})).unwrap();
    let content_schema = ContentSchema::Json(schema);
    let valid_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

    let rule = ContentRule::StaticSchema(content_schema);
    let doc = doc_gen(valid_content);
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new()
            .with_content(vec![1, 2, 3])
            .build()
    }
    => true
    ;
    "expected not nil content"
)]
#[test_case(
    || {
        Builder::new()
            .with_content(vec![])
            .build()
    }
    => true
    ;
    "expected not nil empty content"
)]
#[test_case(
    || {
        Builder::new()
            .build()
    }
    => false
    ;
    "not expected nil content"
)]
fn template_rule_not_nil_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = ContentRule::NotNil;
    let doc = doc_gen();
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new()
            .build()
    }
    => true
    ;
    "expected nil content"
)]
#[test_case(
    || {
        Builder::new()
            .with_content(vec![1, 2, 3])
            .build()
    }
    => false
    ;
    "non expected not nil content"
)]
#[test_case(
    || {
        Builder::new()
            .with_content(vec![])
            .build()
    }
    => false
    ;
    "non expected not nil empty"
)]
fn template_rule_nil_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = ContentRule::Nil;
    let doc = doc_gen();
    rule.check_inner(&doc)
}
