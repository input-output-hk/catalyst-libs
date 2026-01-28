use crate::{
    CatalystSignedDocument, builder, providers::tests::TestCatalystProvider,
    tests_utils::create_dummy_admin_key_pair,
};

pub fn brand_parameters_doc(
    template: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());

    builder::brand_parameters_doc(
        &template.doc_ref()?,
        &serde_json::json!({}),
        &sk.into(),
        kid,
        None,
    )
}
