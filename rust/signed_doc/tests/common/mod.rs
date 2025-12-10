#![allow(dead_code, unused_imports)]

pub mod brand_parameters;
pub mod brand_parameters_form_template;
pub mod campaign_parameters;
pub mod campaign_parameters_form_template;
pub mod category_parameters;
pub mod category_parameters_form_template;
pub mod contest_ballot;
pub mod contest_ballot_checkpoint;
pub mod contest_parameters;
pub mod contest_parameters_form_template;
pub mod proposal;
pub mod proposal_comment;
pub mod proposal_comment_form_template;
pub mod proposal_form_template;
pub mod proposal_submission_action;
pub mod rep_profile;
pub mod rep_profile_form_template;

use std::str::FromStr;

pub use brand_parameters::brand_parameters_doc;
pub use brand_parameters_form_template::brand_parameters_form_template_doc;
pub use campaign_parameters::campaign_parameters_doc;
pub use campaign_parameters_form_template::campaign_parameters_form_template_doc;
use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::{
    catalyst_id::role_index::RoleId,
    uuid::{UuidV4, UuidV7},
};
pub use category_parameters::category_parameters_doc;
pub use category_parameters_form_template::category_parameters_form_template_doc;
pub use contest_ballot::contest_ballot_doc;
pub use contest_ballot_checkpoint::contest_ballot_checkpoint_doc;
pub use contest_parameters::contest_parameters_doc;
pub use contest_parameters_form_template::contest_parameters_form_template_doc;
pub use proposal::proposal_doc;
pub use proposal_comment::proposal_comment_doc;
pub use proposal_comment_form_template::proposal_comment_form_template_doc;
pub use proposal_form_template::proposal_form_template_doc;
pub use proposal_submission_action::proposal_submission_action_doc;
pub use rep_profile_form_template::rep_profile_form_template_doc;

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

// If `None` make `CatalystId` as admin
pub fn create_dummy_key_pair(
    role_index: Option<RoleId>
) -> (ed25519_dalek::SigningKey, CatalystId) {
    let sk = create_signing_key();
    let kid = if let Some(role_index) = role_index {
        CatalystId::new("cardano", None, sk.verifying_key()).with_role(role_index)
    } else {
        CatalystId::new("cardano", None, sk.verifying_key()).as_admin()
    };
    (sk, kid)
}

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}

#[allow(clippy::expect_used)]
pub fn create_dummy_doc_ref() -> DocumentRef {
    let test_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "id": UuidV7::new().to_string(),
            "ver": UuidV7::new().to_string(),
            "type": UuidV4::new().to_string(),
            "content-type": ContentType::Json,
        }))
        .expect("Should create metadata")
        .with_json_content(&serde_json::json!({"test": "content"}))
        .expect("Should set content")
        .build()
        .expect("Should build document");

    test_doc.doc_ref().expect("Should generate DocumentRef")
}
