#![allow(dead_code)]

use std::str::FromStr;

use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

pub fn test_metadata() -> (UuidV7, UuidV4, serde_json::Value) {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    let metadata_fields = serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": uuid_v4.to_string(),
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
        "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
        "template": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
        "section": "$".to_string(),
        "collabs": vec!["Alex1".to_string(), "Alex2".to_string()],
        "parameters": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
    });

    (uuid_v7, uuid_v4, metadata_fields)
}

pub fn create_dummy_key_pair(
    role_index: RoleId,
) -> anyhow::Result<(
    ed25519_dalek::SigningKey,
    ed25519_dalek::VerifyingKey,
    CatalystId,
)> {
    let sk = create_signing_key();
    let pk = sk.verifying_key();
    let kid = CatalystId::from_str(&format!(
        "id.catalyst://cardano/{}/{role_index}/0",
        base64_url::encode(pk.as_bytes())
    ))?;

    Ok((sk, pk, kid))
}

pub fn create_dummy_doc(
    doc_type_id: Uuid,
) -> anyhow::Result<(CatalystSignedDocument, UuidV7, UuidV7)> {
    let empty_json = serde_json::to_vec(&serde_json::json!({}))?;

    let doc_id = UuidV7::new();
    let doc_ver = UuidV7::new();

    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "type": doc_type_id,
            "id": doc_id,
            "ver": doc_ver,
            "template": { "id": doc_id.to_string(), "ver": doc_ver.to_string() }
        }))?
        .with_decoded_content(empty_json.clone())
        .build();

    Ok((doc, doc_id, doc_ver))
}

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}

pub fn create_dummy_signed_doc(
    metadata: serde_json::Value, content: Vec<u8>, with_role_index: RoleId,
) -> anyhow::Result<(
    CatalystSignedDocument,
    ed25519_dalek::VerifyingKey,
    CatalystId,
)> {
    let (sk, pk, kid) = create_dummy_key_pair(with_role_index)?;

    let signed_doc = Builder::new()
        .with_decoded_content(content)
        .with_json_metadata(metadata)?
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)?
        .build();

    Ok((signed_doc, pk, kid))
}
