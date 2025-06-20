#![allow(dead_code)]

use std::str::FromStr;

use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;

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

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}
