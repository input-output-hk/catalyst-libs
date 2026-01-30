//! Reusable functionality for building and signing documents
//! # WARNING
//! FOR TESTING PURPOSES ONLY, DON'T USE IN PRODUCTION CODE

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items
)]

pub mod brand_parameters;
pub mod brand_parameters_form_template;
pub mod campaign_parameters;
pub mod campaign_parameters_form_template;
pub mod category_parameters;
pub mod category_parameters_form_template;
pub mod contest_ballot;
pub mod contest_ballot_checkpoint;
pub mod contest_delegation;
pub mod contest_parameters;
pub mod contest_parameters_form_template;
pub mod proposal;
pub mod proposal_comment;
pub mod proposal_comment_form_template;
pub mod proposal_form_template;
pub mod proposal_submission_action;
pub mod rep_nomination;
pub mod rep_nomination_form_template;
pub mod rep_profile;
pub mod rep_profile_form_template;

use std::sync::LazyLock;

pub use brand_parameters::brand_parameters_doc;
pub use brand_parameters_form_template::brand_parameters_form_template_doc;
pub use campaign_parameters::campaign_parameters_doc;
pub use campaign_parameters_form_template::campaign_parameters_form_template_doc;
pub use category_parameters::category_parameters_doc;
pub use category_parameters_form_template::category_parameters_form_template_doc;
pub use contest_ballot::contest_ballot_doc;
pub use contest_ballot_checkpoint::contest_ballot_checkpoint_doc;
pub use contest_delegation::{contest_delegation_by_representative_doc, contest_delegation_doc};
pub use contest_parameters::contest_parameters_doc;
pub use contest_parameters_form_template::contest_parameters_form_template_doc;
pub use proposal::proposal_doc;
pub use proposal_comment::proposal_comment_doc;
pub use proposal_comment_form_template::proposal_comment_form_template_doc;
pub use proposal_form_template::proposal_form_template_doc;
pub use proposal_submission_action::proposal_submission_action_doc;
pub use rep_nomination::rep_nomination_doc;
pub use rep_nomination_form_template::rep_nomination_form_template_doc;
pub use rep_profile::rep_profile_doc;
pub use rep_profile_form_template::rep_profile_form_template_doc;

use crate::{
    CatalystSignedDocument, ContentType, DocumentRef,
    builder::Builder,
    catalyst_id::{CatalystId, role_index::RoleId},
    providers::tests::TestCatalystProvider,
    uuid::{UuidV4, UuidV7},
    validator::Validator,
};

#[allow(clippy::missing_errors_doc)]
pub fn get_doc_kid_and_sk(
    provider: &TestCatalystProvider,
    doc: &CatalystSignedDocument,
    i: usize,
) -> anyhow::Result<(ed25519_dalek::SigningKey, CatalystId)> {
    let doc_kids = doc.authors();
    let kid = doc_kids
        .get(i)
        .ok_or(anyhow::anyhow!("does not have a kid at index '{i}'"))?;
    let sk = provider.get_sk(kid).ok_or(anyhow::anyhow!(
        "cannot find a corresponding signing key to the kid '{kid}'"
    ))?;
    Ok((sk.clone(), kid.clone()))
}

#[must_use]
pub fn create_dummy_key_pair(role_index: RoleId) -> (ed25519_dalek::SigningKey, CatalystId) {
    let sk = create_signing_key();
    let kid = CatalystId::new("cardano", None, sk.verifying_key()).with_role(role_index);
    (sk, kid)
}

#[must_use]
pub fn create_dummy_admin_key_pair() -> (ed25519_dalek::SigningKey, CatalystId) {
    let sk = create_signing_key();
    let kid = CatalystId::new("cardano", None, sk.verifying_key()).as_admin();
    (sk, kid)
}

#[must_use]
pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}

#[must_use]
pub fn create_dummy_doc_ref() -> DocumentRef {
    let test_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "id": UuidV7::new().to_string(),
            "ver": UuidV7::new().to_string(),
            "type": UuidV4::new().to_string(),
            "content-type": ContentType::Json,
        }))
        .expect("Must be valid metadata fields")
        .with_json_content(&serde_json::json!({"test": "content"}))
        .expect("Must be valid JSON object")
        .build()
        .expect("Must be valid document");

    test_doc.doc_ref().expect("Must be valid DocumentRef")
}

pub fn build_verify_and_publish(
    provider: &mut TestCatalystProvider,
    gen_fn: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>,
) -> anyhow::Result<CatalystSignedDocument> {
    static VALIDATOR: LazyLock<Validator> = LazyLock::new(|| Validator::new());

    let doc = gen_fn(provider)?;
    VALIDATOR.validate(&doc, provider)?;
    anyhow::ensure!(
        !doc.report().is_problematic(),
        "Invalid document, report: {:?}",
        doc.report()
    );

    provider.add_document(&doc)?;
    Ok(doc)
}

#[must_use]
pub fn create_key_pair_and_publish(
    provider: &mut TestCatalystProvider,
    gen_fn: impl FnOnce() -> (ed25519_dalek::SigningKey, CatalystId),
) -> (ed25519_dalek::SigningKey, CatalystId) {
    let (sk, kid) = gen_fn();
    provider.add_sk(kid.clone(), sk.clone());
    (sk, kid)
}
