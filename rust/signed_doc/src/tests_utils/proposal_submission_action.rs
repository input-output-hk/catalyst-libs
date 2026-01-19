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
        &serde_json::json!({
            "action": "final"
        }),
        linked,
        parameters,
        &builder::ed25519::Ed25519SigningKey::Common(sk),
        kid,
        None,
    )
}
