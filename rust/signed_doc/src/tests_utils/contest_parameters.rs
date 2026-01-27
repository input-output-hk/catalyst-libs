use chrono::{Duration, Utc};

use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

#[must_use]
pub fn contest_parameters_default_content() -> serde_json::Value {
    serde_json::json!({
        "start": Utc::now(),
        "end": Utc::now().checked_add_signed(Duration::minutes(5)),
        "snapshot": Utc::now(),
        "election_public_key": "0000000000000000000000000000000000000000000000000000000000000000",
        "options": ["Yes", "No", "Abstain"]
    })
}

pub fn contest_parameters_doc(
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());
    let content = contest_parameters_default_content();
    builder::contest_parameters_doc(&template.doc_ref()?, &parameters.doc_ref()?, &content, &sk.into(), kid, None)
}
