use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    Builder, CatalystSignedDocument, ContentEncoding, ContentType,
    catalyst_id::role_index::RoleId,
    doc_types,
    providers::tests::TestCatalystProvider,
    tests_utils::{create_dummy_key_pair, get_doc_kid_and_sk},
    uuid::UuidV7,
};

#[allow(clippy::missing_errors_doc)]
pub fn contest_delegation_by_representative_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = get_doc_kid_and_sk(provider, ref_doc, 0)
        .map(|(sk, kid)| (sk, kid.with_role(RoleId::Role0)))
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

    contest_delegation_doc_inner(ref_doc, parameters_doc, kid, sk)
}

#[allow(clippy::missing_errors_doc)]
pub fn contest_delegation_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
    provider.add_sk(kid.clone(), sk.clone());

    contest_delegation_doc_inner(ref_doc, parameters_doc, kid, sk)
}

#[allow(clippy::missing_errors_doc)]
fn contest_delegation_doc_inner(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    kid: CatalystId,
    sk: ed25519_dalek::SigningKey,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let parameters_ref = parameters_doc.doc_ref()?;
    let ref_ref = ref_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_DELEGATION.clone(),
            "id": id,
            "ver": id,
            "ref": [ref_ref],
            "parameters": [parameters_ref],
        }))?
        .with_json_content(&serde_json::json!({"weights" : []}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
