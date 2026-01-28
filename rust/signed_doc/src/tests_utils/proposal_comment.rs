use crate::{
    CatalystSignedDocument, builder, catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair,
};

pub fn proposal_comment_doc(
    linked: &CatalystSignedDocument,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
    provider.add_sk(kid.clone(), sk.clone());
    builder::proposal_comment_doc(
        &linked.doc_ref()?,
        &template.doc_ref()?,
        &parameters.doc_ref()?,
        &serde_json::json!({}),
        &sk.into(),
        kid,
        None,
    )
}
