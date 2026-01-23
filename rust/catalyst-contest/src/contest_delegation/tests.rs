//! Test for 'Contest Delegation' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_doc_and_publish,
        contest_delegation_by_representative_doc, contest_delegation_doc,
        contest_parameters::contest_parameters_default_content, contest_parameters_doc,
        contest_parameters_form_template_doc, create_dummy_admin_key_pair,
        create_key_pair_and_publish, rep_nomination_doc, rep_nomination_form_template_doc,
        rep_profile_doc, rep_profile_form_template_doc,
    },
    validator::Validator,
    *,
};
use chrono::{Duration, Utc};
use test_case::test_case;

use crate::contest_delegation::{ContestDelegation, rule::ContestDelegationRule};

#[test_case(
    |p| {
        let template = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| rep_profile_form_template_doc(&brand, p))?;
        let rep_profile = build_doc_and_publish(p, |p| rep_profile_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let contest = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| rep_nomination_form_template_doc(&contest, p))?;
        let rep_nomination = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        let _delegation_by_representative = build_doc_and_publish(p, |p| contest_delegation_by_representative_doc(&rep_nomination, &contest, p))?;
        contest_delegation_doc(&rep_nomination, &contest, p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(1)));
        content["end"] = serde_json::json!(Utc::now().checked_add_signed(Duration::hours(5)));

        let template = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| rep_profile_form_template_doc(&brand, p))?;
        let rep_profile = build_doc_and_publish(p, |p| rep_profile_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let contest = build_doc_and_publish(p, |_|  builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| rep_nomination_form_template_doc(&contest, p))?;
        let rep_nomination = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        let _delegation_by_representative = build_doc_and_publish(p, |p| contest_delegation_by_representative_doc(&rep_nomination, &contest, p))?;
        contest_delegation_doc(&rep_nomination, &contest, p)
    }
    => false
    ;
    "failed timeline check, too early"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
                let mut content = contest_parameters_default_content();
        content["start"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(5)));
        content["end"] = serde_json::json!(Utc::now().checked_sub_signed(Duration::hours(1)));

        let template = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| rep_profile_form_template_doc(&brand, p))?;
        let rep_profile = build_doc_and_publish(p, |p| rep_profile_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let contest = build_doc_and_publish(p, |_|  builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| rep_nomination_form_template_doc(&contest, p))?;
        let rep_nomination = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        let _delegation_by_representative = build_doc_and_publish(p, |p| contest_delegation_by_representative_doc(&rep_nomination, &contest, p))?;
        contest_delegation_doc(&rep_nomination, &contest, p)
    }
    => false
    ;
    "failed timeline check, too old"
)]
#[test_case(
    |p| {
        let template = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| rep_profile_form_template_doc(&brand, p))?;
        let rep_profile = build_doc_and_publish(p, |p| rep_profile_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let contest = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| rep_nomination_form_template_doc(&contest, p))?;
        let rep_nomination = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        contest_delegation_doc(&rep_nomination, &contest, p)
    }
    => false
    ;
    "missing delegation by representative"
)]
#[test_case(
    |p| {
        let template = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&template, p))?;
        let template = build_doc_and_publish(p, |p| rep_profile_form_template_doc(&brand, p))?;
        let rep_profile = build_doc_and_publish(p, |p| rep_profile_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let contest = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| rep_nomination_form_template_doc(&contest, p))?;
        let rep_nomination = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        let _delegation_by_representative = build_doc_and_publish(p, |p| contest_delegation_by_representative_doc(&rep_nomination, &contest, p))?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let rep_nomination_latest = build_doc_and_publish(p, |p| rep_nomination_doc(&template, &rep_profile, &contest, p))?;
        let _delegation_by_representative_2 = build_doc_and_publish(p, |p| contest_delegation_by_representative_doc(&rep_nomination_latest, &contest, p))?;
        contest_delegation_doc(&rep_nomination, &contest, p)
    }
    => false
    ;
    "not the latest nomination reference"
)]
#[allow(clippy::unwrap_used)]
fn contest_delegation(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut p = TestCatalystProvider::default();
    let doc = doc_gen(&mut p).unwrap();

    let mut validator = Validator::new();
    validator
        .extend_rules_per_document(doc_types::CONTEST_DELEGATION.clone(), ContestDelegationRule);

    validator.validate(&doc, &p).unwrap();
    println!("{:?}", doc.report());
    let is_valid = !doc.report().is_problematic();

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report
    let doc = doc_gen(&mut p).unwrap();
    let contest_delegation = ContestDelegation::new(&doc, &p).unwrap();
    println!("{:?}", contest_delegation.report());
    assert_eq!(is_valid, !contest_delegation.report().is_problematic());

    is_valid
}
