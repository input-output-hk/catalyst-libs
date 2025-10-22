#![allow(dead_code, unused_imports)]

pub mod brand_parameters;
pub mod brand_parameters_form_template;
pub mod campaign_parameters;
pub mod campaign_parameters_form_template;
pub mod category_parameters;
pub mod category_parameters_form_template;
pub mod proposal;
pub mod proposal_comment;
pub mod proposal_comment_form_template;
pub mod proposal_form_template;
pub mod proposal_submission_action;

use std::str::FromStr;

pub use brand_parameters::brand_parameters_doc;
pub use brand_parameters_form_template::brand_parameters_form_template_doc;
pub use campaign_parameters::campaign_parameters_doc;
pub use campaign_parameters_form_template::campaign_parameters_form_template_doc;
use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;
pub use category_parameters::category_parameters_doc;
pub use category_parameters_form_template::category_parameters_form_template_doc;
pub use proposal::proposal_doc;
pub use proposal_comment::proposal_comment_doc;
pub use proposal_comment_form_template::proposal_comment_form_template_doc;
pub use proposal_form_template::proposal_form_template_doc;
pub use proposal_submission_action::proposal_submission_action_doc;

pub fn create_dummy_key_pair(
    role_index: RoleId
) -> anyhow::Result<(ed25519_dalek::SigningKey, CatalystId)> {
    let sk = create_signing_key();
    let kid = CatalystId::from_str(&format!(
        "id.catalyst://cardano/{}/{role_index}/0",
        base64_url::encode(sk.verifying_key().as_bytes())
    ))?;

    Ok((sk, kid))
}

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}
