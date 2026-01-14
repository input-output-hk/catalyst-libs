use crate::{
    CatalystSignedDocument, builder, catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider, tests_utils::get_doc_kid_and_sk,
};

pub fn rep_nomination_doc(
    template: &CatalystSignedDocument,
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = get_doc_kid_and_sk(provider, linked, 0)
        .map(|(sk, kid)| (sk, kid.with_role(RoleId::DelegatedRepresentative)))
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

    builder::rep_nomination_doc(
        &serde_json::json!({}),
        linked,
        template,
        parameters,
        &sk,
        kid,
        None,
    )
}
