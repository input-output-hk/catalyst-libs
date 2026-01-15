use chrono::{Duration, Utc};

use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

pub fn contest_parameters_doc(
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());
    let content = serde_json::json!({
        "start": Utc::now(),
        "end": Utc::now() + Duration::minutes(5),
    });

    builder::contest_parameters_doc(
        &content,
        template,
        parameters,
        &builder::ed25519::Ed25519SigningKey::Common(sk),
        kid,
        None,
    )
}
