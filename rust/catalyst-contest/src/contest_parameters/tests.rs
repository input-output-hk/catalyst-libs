//! Test for 'Contest Delegation' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_parameters>

use catalyst_signed_doc::{
    builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_doc_and_publish,
        contest_parameters_doc, contest_parameters_form_template_doc, create_dummy_admin_key_pair,
        create_key_pair_and_publish,
    },
    validator::Validator,
    *,
};
use chrono::Utc;
use test_case::test_case;

use crate::contest_parameters::{ContestParameters, rule::ContestParametersRule};

#[test_case(
    |p| {
        let template = build_doc_and_publish(p, |p| brand_parameters_form_template_doc(p))?;
        let parameters = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&parameters, p))?;
        contest_parameters_doc(&template, &parameters, p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let time = Utc::now();
        let content = serde_json::json!({
            "start": time,
            "end": time,
        });

        let template = build_doc_and_publish(p, |p| brand_parameters_form_template_doc(p))?;
        let parameters = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&parameters, p))?;
        builder::contest_parameters_doc(&content, &template, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None)
    }
    => false
    ;
    "invalid content, end date must be after the start date"
)]
#[allow(clippy::unwrap_used)]
fn contest_parameters(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut p = TestCatalystProvider::default();
    let doc = doc_gen(&mut p).unwrap();

    let mut validator = Validator::new();
    validator
        .extend_rules_per_document(doc_types::CONTEST_PARAMETERS.clone(), ContestParametersRule);

    let is_valid = validator.validate(&doc, &p).unwrap();
    println!("{:?}", doc.report());
    assert_eq!(is_valid, !doc.report().is_problematic());

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report
    let doc = doc_gen(&mut p).unwrap();
    let contest_delegation = ContestParameters::new(&doc, &p).unwrap();
    println!("{:?}", contest_delegation.report());
    assert_eq!(is_valid, !contest_delegation.report().is_problematic());

    is_valid
}
