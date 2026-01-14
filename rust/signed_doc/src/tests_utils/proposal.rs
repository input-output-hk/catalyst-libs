use crate::{
    CatalystSignedDocument, builder, catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair,
};

pub fn proposal_doc(
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_key_pair(RoleId::Proposer);
    provider.add_sk(kid.clone(), sk.clone());
    builder::proposal_doc(&serde_json::json!({}), template, parameters, &sk, kid, None)
}
