use std::str::FromStr;

use crate::{
    CatalystId, Ed25519SigningKey, Error, Json, Result, Uuid,
    catalyst_signed_doc::{self, CatalystSignedDocument},
};

#[uniffi::export]
fn brand_parameters_form_template_doc(
    content: Json,
    sk: Ed25519SigningKey,
    kid: CatalystId,
    id: Option<Uuid>,
) -> Result<CatalystSignedDocument> {
    let content =
        serde_json::Value::from_str(content.as_str()).map_err(|e| Error::Anyhow(e.into()))?;
    let sk = catalyst_signed_doc_lib::builder::ed25519::Ed25519SigningKey::from_str(sk.as_str())
        .map_err(|e| Error::Anyhow(e))?;
    let kid = catalyst_signed_doc_lib::catalyst_id::CatalystId::from_str(kid.as_str())
        .map_err(|e| Error::Anyhow(e.into()))?;
    let id = id
        .map(|id| catalyst_signed_doc_lib::uuid::UuidV7::from_str(id.as_str()))
        .transpose()
        .map_err(|e| Error::Anyhow(e.into()))?;

    catalyst_signed_doc_lib::builder::brand_parameters_form_template_doc(&content, &sk, kid, id)
        .map(CatalystSignedDocument)
        .map_err(|e| Error::Anyhow(e.into()))
}
