use crate::{
    CatalystSignedDocument, builder, catalyst_id::role_index::RoleId,
    providers::tests::TestCatalystProvider, tests_utils::get_doc_kid_and_sk,
};

pub fn proposal_submission_action_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = get_doc_kid_and_sk(provider, linked, 0)
        .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

    builder::proposal_submission_action_doc(
        &linked.doc_ref()?,
        &parameters.doc_ref()?,
        &serde_json::json!({
            "action": "final"
        }),
        &sk.into(),
        kid,
        None,
    )
}
