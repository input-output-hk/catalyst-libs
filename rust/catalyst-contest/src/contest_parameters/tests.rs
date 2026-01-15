//! Test for 'Contest Delegation' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_parameters>

use catalyst_signed_doc::{
    builder::contest_parameters_doc,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc,
        contest_parameters_form_template_doc, create_dummy_admin_key_pair,
        create_key_pair_and_publish,
    },
    validator::Validator,
    *,
};
use test_case::test_case;
use chrono::Utc;

use crate::contest_parameters::{ContestParameters, rule::ContestParametersRule};

#[test_case(
    |provider| {
        let (sk, kid) = create_key_pair_and_publish(provider, create_dummy_admin_key_pair);
        let contest = serde_json::json!({
            "start": Utc::now(),
            "end": Utc::now(),
        });

        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_parameters_doc(&contest, &template, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let (sk, kid) = create_key_pair_and_publish(provider, create_dummy_admin_key_pair);
        let time = Utc::now();
        let contest = serde_json::json!({
            "start": time,
            "end": time,
        });

        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_parameters_doc(&contest, &template, &parameters, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None)
    }
    => false
    ;
    "invalid content, end date must be after the start date"
)]
#[allow(clippy::unwrap_used)]
fn contest_parameters(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator
        .extend_rules_per_document(doc_types::CONTEST_PARAMETERS.clone(), ContestParametersRule);

    let is_valid = validator.validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report
    let doc = doc_gen(&mut provider).unwrap();
    let contest_delegation = ContestParameters::new(&doc, &provider).unwrap();
    assert_eq!(is_valid, !contest_delegation.report().is_problematic());
    println!("{:?}", contest_delegation.report());

    is_valid
}
