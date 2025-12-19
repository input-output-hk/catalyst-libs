use catalyst_types::catalyst_id::role_index::RoleId;
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, tests_utils::create_dummy_key_pair,
};

#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Collaborators(
                    vec![create_dummy_key_pair(RoleId::Role0).1].into()
                ))
            .build()
    }
    => true
    ;
    "valid 'collaborators' field present"
)]
#[test_case(
    || {
        Builder::new().build()
    }
    => true
    ;
    "missing 'collaborators' field"
)]
#[tokio::test]
fn section_rule_specified_optional_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = CollaboratorsRule::Specified { optional: true };

    let doc = doc_gen();
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Collaborators(
                    vec![create_dummy_key_pair(RoleId::Role0).1].into()
                ))
            .build()
    }
    => true
    ;
    "valid 'collaborators' field present"
)]
#[test_case(
    || {
        Builder::new().build()
    }
    => false
    ;
    "missing 'collaborators' field"
)]
#[tokio::test]
fn section_rule_specified_not_optional_test(
    doc_gen: impl FnOnce() -> CatalystSignedDocument
) -> bool {
    let rule = CollaboratorsRule::Specified { optional: false };

    let doc = doc_gen();
    rule.check_inner(&doc)
}

#[test_case(
    || {
        Builder::new().build()
    }
    => true
    ;
    "missing 'collaborators' field"
)]
#[test_case(
    || {
        Builder::new()
            .with_metadata_field(SupportedField::Collaborators(
                    vec![create_dummy_key_pair(RoleId::Role0).1].into()
                ))
            .build()
    }
    => false
    ;
    "valid 'collaborators' field present"
)]
#[tokio::test]
fn section_rule_not_specified_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
    let rule = CollaboratorsRule::NotSpecified;

    let doc = doc_gen();
    rule.check_inner(&doc)
}
