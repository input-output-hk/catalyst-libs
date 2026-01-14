use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

pub fn contest_ballot_checkpoint_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());
    builder::contest_ballot_checkpoint_doc(linked, parameters, sk, kid, None)
}
