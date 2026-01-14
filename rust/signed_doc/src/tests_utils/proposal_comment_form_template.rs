use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

pub fn proposal_comment_form_template_doc(
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());
    builder::proposal_comment_form_template_doc(
        &serde_json::json!({}),
        parameters,
        &builder::ed25519::Ed25519SigningKey::Common(sk),
        kid,
        None,
    )
}
