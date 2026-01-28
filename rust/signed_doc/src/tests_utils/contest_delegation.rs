use crate::{
    CatalystSignedDocument, builder,
    catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider,
    tests_utils::{create_dummy_key_pair, get_doc_kid_and_sk},
};

pub fn contest_delegation_by_representative_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = get_doc_kid_and_sk(provider, linked, 0)
        .map(|(sk, kid)| (sk, kid.with_role(RoleId::Role0)))
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;
    builder::contest_delegation_doc(
        &linked.doc_ref()?,
        &parameters.doc_ref()?,
        &serde_json::json!({"weights" : []}),
        &sk.into(),
        kid,
        None,
    )
}

pub fn contest_delegation_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
    provider.add_sk(kid.clone(), sk.clone());
    builder::contest_delegation_doc(
        &linked.doc_ref()?,
        &parameters.doc_ref()?,
        &serde_json::json!({"weights" : []}),
        &sk.into(),
        kid,
        None,
    )
}
