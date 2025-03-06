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

pub struct Provider(pub anyhow::Result<Option<ed25519_dalek::VerifyingKey>>);

impl providers::VerifyingKeyProvider for Provider {
    async fn try_get_key(
        &self, _kid: &IdUri,
    ) -> anyhow::Result<Option<ed25519_dalek::VerifyingKey>> {
        let res = self.0.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(*res)
    }
}