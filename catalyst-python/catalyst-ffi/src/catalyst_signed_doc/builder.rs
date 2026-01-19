use std::str::FromStr;

use crate::{
    CatalystId, Ed25519SigningKey, Error, Json, Result, Uuid,
    catalyst_signed_doc::CatalystSignedDocument,
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

#[uniffi::export]
fn brand_parameters(
    content: Json,
    template: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::brand_parameters_doc(&content, &template.0, &sk, kid, id)
        .map(CatalystSignedDocument)
        .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn campaign_parameters_form_template_doc(
    content: Json,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::campaign_parameters_form_template_doc(
        &content,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn campaign_parameters_doc(
    content: Json,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::campaign_parameters_doc(
        &content,
        &template.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn category_parameters_form_template_doc(
    content: Json,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::category_parameters_form_template_doc(
        &content,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn category_parameters_doc(
    content: Json,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::category_parameters_doc(
        &content,
        &template.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn contest_parameters_form_template_doc(
    content: Json,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::contest_parameters_form_template_doc(
        &content,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn contest_parameters_doc(
    content: Json,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::contest_parameters_doc(
        &content,
        &template.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn proposal_comment_form_template_doc(
    content: Json,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::proposal_comment_form_template_doc(
        &content,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn proposal_comment_doc(
    content: Json,
    linked: &CatalystSignedDocument,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::proposal_comment_doc(
        &content,
        &linked.0,
        &template.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn proposal_form_template_doc(
    content: Json,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::proposal_form_template_doc(
        &content,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn proposal_submission_action_doc(
    content: Json,
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::proposal_submission_action_doc(
        &content,
        &linked.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}

#[uniffi::export]
fn proposal_doc(
    content: Json,
    template: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
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

    catalyst_signed_doc_lib::builder::proposal_doc(
        &content,
        &template.0,
        &parameters.0,
        &sk,
        kid,
        id,
    )
    .map(CatalystSignedDocument)
    .map_err(|e| Error::Anyhow(e.into()))
}
