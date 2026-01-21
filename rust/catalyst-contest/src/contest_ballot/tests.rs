//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, builder, doc_types,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_doc_and_publish,
        contest_ballot_doc, contest_parameters_doc, contest_parameters_form_template_doc,
        create_dummy_admin_key_pair, create_key_pair_and_publish, proposal_doc,
        proposal_form_template_doc,
    },
    validator::Validator,
};
use chrono::{Duration, Utc};
use test_case::test_case;

use crate::{ContestBallotRule, contest_ballot::ballot::ContestBallot};

#[test_case(
    |p| {
        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |p| contest_parameters_doc(&template, &brand, p))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let content = serde_json::json!({
            "start": Utc::now().checked_add_signed(Duration::hours(1)),
            "end": Utc::now().checked_add_signed(Duration::hours(5)),
        });

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |_| builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => false
    ;
    "failed timeline check, too early"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let content = serde_json::json!({
            "start": Utc::now().checked_sub_signed(Duration::hours(5)),
            "end": Utc::now().checked_sub_signed(Duration::hours(1)),
        });

        let brand = build_doc_and_publish(p, brand_parameters_form_template_doc)?;
        let brand = build_doc_and_publish(p, |p| brand_parameters_doc(&brand, p))?;
        let template = build_doc_and_publish(p, |p| contest_parameters_form_template_doc(&brand, p))?;
        let parameters = build_doc_and_publish(p, |_| builder::contest_parameters_doc(&content, &template, &brand, &builder::ed25519::Ed25519SigningKey::Common(sk), kid, None))?;
        let template = build_doc_and_publish(p, |p| proposal_form_template_doc(&parameters, p))?;
        let proposal = build_doc_and_publish(p, |p| proposal_doc(&template, &parameters, p))?;
        contest_ballot_doc(&proposal, &parameters, p)
    }
    => false
    ;
    "failed timeline check, too old"
)]
fn contest_ballot(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator.extend_rules_per_document(doc_types::CONTEST_BALLOT.clone(), ContestBallotRule {});

    validator.validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    let is_valid = !doc.report().is_problematic();

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report.
    let doc = doc_gen(&mut provider).unwrap();
    let contest_ballot = ContestBallot::new(&doc, &provider).unwrap();
    println!("{:?}", contest_ballot.report());
    assert_eq!(is_valid, !contest_ballot.report().is_problematic());

    is_valid
}
