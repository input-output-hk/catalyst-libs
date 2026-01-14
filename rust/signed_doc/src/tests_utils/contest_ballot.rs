use catalyst_types::catalyst_id::role_index::RoleId;

use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_key_pair,
};

pub fn contest_ballot_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
    provider.add_sk(kid.clone(), sk.clone());
    builder::contest_ballot_doc(linked, parameters, sk, kid, None)
}
