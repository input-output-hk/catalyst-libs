//! Integration test for contest ballot checkpoint document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot_checkpoint>

use catalyst_signed_doc::{
    builder::Builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
        campaign_parameters_form_template_doc, category_parameters_doc,
        category_parameters_form_template_doc, contest_ballot_checkpoint_doc, contest_ballot_doc,
        contest_parameters_doc, contest_parameters_form_template_doc, create_dummy_admin_key_pair,
        create_dummy_key_pair, proposal_doc, proposal_form_template_doc,
    },
    validator::Validator,
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_ballot_checkpoint_doc(&linked_doc, &parameters, provider)
    }
    => true
    ;
    "valid document with brand 'parameters'"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let campaign = campaign_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let campaign = campaign_parameters_doc(&campaign, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&campaign, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &campaign, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_ballot_checkpoint_doc(&linked_doc, &parameters, provider)
    }
    => true
    ;
    "valid document with campaign 'parameters'"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let campaign = campaign_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let campaign = campaign_parameters_doc(&campaign, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let category = category_parameters_form_template_doc(&campaign, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let category = category_parameters_doc(&category, &campaign, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&category, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &category, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_ballot_checkpoint_doc(&linked_doc, &parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters'"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let linked_ref = linked_doc.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;
        let chain = Chain::new(0, None);

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "ref": [linked_ref],
                "parameters": [parameters_ref],
                "chain": chain,
            }))?
            .with_cbor_content(1)?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "wrong role"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let linked_ref = linked_doc.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;
        let chain = Chain::new(0, None);

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "ref": [linked_ref],
                "parameters": [parameters_ref],
                "chain": chain,
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
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let linked_ref = linked_doc.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;
        let chain = Chain::new(0, None);

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "ref": [linked_ref],
                "parameters": [parameters_ref],
                "chain": chain,
            }))?
            .with_cbor_content(1)?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let linked_ref = linked_doc.doc_ref()?;
        let chain = Chain::new(0, None);

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "ref": [linked_ref],
                "chain": chain,
            }))?
            .with_cbor_content(1)?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing parameters"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;
        let chain = Chain::new(0, None);

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "parameters": [parameters_ref],
                "chain": chain,
            }))?
            .with_cbor_content(1)?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing ref"
)]
#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let linked_doc = contest_ballot_doc(&proposal, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let linked_ref = linked_doc.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Cbor,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
                "id": id,
                "ver": id,
                "ref": [linked_ref],
                "parameters": [parameters_ref],
            }))?
            .with_cbor_content(1)?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing chain"
)]
#[allow(clippy::unwrap_used)]
fn contest_ballot_checkpoint(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::CONTEST_BALLOT_CHECKPOINT.clone()
    );

    Validator::new().validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
