use test_case::test_case;

use super::*;
use crate::{builder::tests::Builder, metadata::SupportedField};

#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Revocations(
                    vec![].into()
                ))
            .build()
    }
    => true
    ;
    "valid 'revocations' field present"
)]
#[test_case(
    || {
        Builder::new().build()
    }
    => true
    ;
    "missing 'revocations' field"
)]

fn rule_specified_optional_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = RevocationsRule::Specified { optional: true };

    let doc = doc_gen();
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Revocations(
                    vec![].into()
                ))
            .build()
    }
    => true
    ;
    "valid 'revocations' field present"
)]
#[test_case(
    || {
        Builder::new().build()
    }
    => false
    ;
    "missing 'revocations' field"
)]

fn rule_specified_not_optional_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = RevocationsRule::Specified { optional: false };

    let doc = doc_gen();
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new().build()
    }
    => true
    ;
    "missing 'revocations' field"
)]
#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Revocations(
                    vec![].into()
                ))
            .build()
    }
    => false
    ;
    "valid 'revocations' field present"
)]

fn rule_not_specified_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = RevocationsRule::NotSpecified;

    let doc = doc_gen();
    rule.check_inner(&doc)
}
