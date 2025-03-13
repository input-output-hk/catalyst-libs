use std::{collections::HashMap, str::FromStr};

use catalyst_signed_doc::*;

pub fn test_metadata() -> (UuidV7, UuidV4, serde_json::Value) {
    let uuid_v7 = UuidV7::new();
    let uuid_v4 = UuidV4::new();

    let metadata_fields = serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": uuid_v4.to_string(),
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": {"id": uuid_v7.to_string()},
        "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
        "template": {"id": uuid_v7.to_string()},
        "section": "$".to_string(),
        "collabs": vec!["Alex1".to_string(), "Alex2".to_string()],
        "campaign_id": {"id": uuid_v7.to_string()},
        "election_id":  uuid_v4.to_string(),
        "brand_id":  {"id": uuid_v7.to_string()},
        "category_id": {"id": uuid_v7.to_string()},
    });

    (uuid_v7, uuid_v4, metadata_fields)
}

pub fn create_dummy_doc(doc_type_id: Uuid) -> anyhow::Result<(CatalystSignedDocument, UuidV7)> {
    let empty_json = serde_json::to_vec(&serde_json::json!({}))?;

    let doc_id = UuidV7::new();

    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "type": doc_type_id,
            "content-type": ContentType::Json.to_string(),
            "template": { "id": doc_id.to_string() }
        }))?
        .with_decoded_content(empty_json.clone())
        .build();

    Ok((doc, doc_id))
}

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}

pub fn create_dummy_signed_doc(
    with_metadata: Option<serde_json::Value>,
) -> anyhow::Result<(CatalystSignedDocument, ed25519_dalek::VerifyingKey)> {
    let sk = create_signing_key();
    let content = serde_json::to_vec(&serde_json::Value::Null)?;
    let (_, _, metadata) = test_metadata();
    let pk = sk.verifying_key();
    let kid_str = format!(
        "id.catalyst://cardano/{}/0/0",
        base64_url::encode(pk.as_bytes())
    );
    let kid = IdUri::from_str(&kid_str)?;

    let signed_doc = Builder::new()
        .with_decoded_content(content)
        .with_json_metadata(with_metadata.unwrap_or(metadata))?
        .add_signature(sk.to_bytes(), kid.clone())?
        .build();

    Ok((signed_doc, pk))
}

pub struct DummyVerifyingKeyProvider(pub anyhow::Result<Option<ed25519_dalek::VerifyingKey>>);

impl providers::VerifyingKeyProvider for DummyVerifyingKeyProvider {
    async fn try_get_key(
        &self, _kid: &IdUri,
    ) -> anyhow::Result<Option<ed25519_dalek::VerifyingKey>> {
        let res = self.0.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(*res)
    }
}

#[derive(Default)]
pub struct DummyCatSignDocProvider(pub HashMap<Uuid, CatalystSignedDocument>);

impl providers::CatalystSignedDocumentProvider for DummyCatSignDocProvider {
    async fn try_get_doc(
        &self, doc_ref: &DocumentRef,
    ) -> anyhow::Result<Option<CatalystSignedDocument>> {
        Ok(self.0.get(&doc_ref.id.uuid()).cloned())
    }
}
