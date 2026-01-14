use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

pub fn rep_nomination_form_template_doc(
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());
    builder::rep_nomination_form_template_doc(&serde_json::json!({}), parameters, sk, kid, None)
}
