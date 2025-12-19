//! Integration test for contest delegation document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, contest_delegation_doc,
        contest_parameters_doc, contest_parameters_form_template_doc, create_dummy_key_pair,
        rep_nomination_doc, rep_nomination_form_template_doc, rep_profile_doc,
        rep_profile_form_template_doc,
    },
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
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_delegation_doc(&rep_nomination, &contest, provider)
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
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Proposer);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = contest.doc_ref()?;
        let ref_ref = rep_nomination.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_DELEGATION.clone(),
                "id": id,
                "ver": id,
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({"weights" : []}))?
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
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = contest.doc_ref()?;
        let ref_ref = rep_nomination.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_DELEGATION.clone(),
                "id": id,
                "ver": id,
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
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = contest.doc_ref()?;
        let ref_ref = rep_nomination.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "type": doc_types::CONTEST_DELEGATION.clone(),
                "id": id,
                "ver": id,
                "ref": [ref_ref],
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({"weights" : []}))?
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
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = contest.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_DELEGATION.clone(),
                "id": id,
                "ver": id,
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({"weights" : []}))?
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
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let ref_ref = rep_nomination.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_DELEGATION.clone(),
                "id": id,
                "ver": id,
                "ref": [ref_ref],
            }))?
            .with_json_content(&serde_json::json!({"weights" : []}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'parameters'"
)]
#[tokio::test]
#[allow(clippy::unwrap_used)]
fn contest_ballot(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::CONTEST_DELEGATION.clone()
    );

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());
    is_valid
}
