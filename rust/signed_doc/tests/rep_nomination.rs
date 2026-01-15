//! Integration test for rep nomination document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/brand_parameters>

use catalyst_signed_doc::{
    builder::Builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, contest_parameters_doc,
        contest_parameters_form_template_doc, get_doc_kid_and_sk, rep_nomination_doc,
        rep_nomination_form_template_doc, rep_profile_doc, rep_profile_form_template_doc,
    },
    validator::Validator,
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        rep_nomination_doc(&template, &rep_profile, &contest, provider)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Role0)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let template_ref = template.doc_ref()?;
        let ref_ref = rep_profile.doc_ref()?;
        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "wrong role"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let template_ref = template.doc_ref()?;
        let ref_ref = rep_profile.doc_ref()?;
        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .empty_content()?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "empty content"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let template_ref = template.doc_ref()?;
        let ref_ref = rep_profile.doc_ref()?;
        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let ref_ref = rep_profile.doc_ref()?;
        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'template'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let template_ref = template.doc_ref()?;
        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'ref'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &rep_profile, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let template_ref = template.doc_ref()?;
        let ref_ref = rep_profile.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::REP_NOMINATION.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "ref": [ref_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'parameters'"
)]
#[allow(clippy::unwrap_used)]
fn test_brand_parameters_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(*doc.doc_type().unwrap(), doc_types::REP_NOMINATION.clone());

    let is_valid = Validator::new().validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());
    is_valid
}
