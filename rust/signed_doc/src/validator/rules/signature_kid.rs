//! Catalyst Signed Document COSE signature `kid` (Catalyst Id) role validation

use catalyst_types::catalyst_id::role_index::RoleIndex;

use crate::CatalystSignedDocument;

///  COSE signature `kid` (Catalyst Id) role validation
pub(crate) struct SignatureKidRule {
    /// expected `RoleIndex` values for the `kid` field
    pub(crate) exp: &'static [RoleIndex],
}

impl SignatureKidRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        let contains_exp_role = doc.kids().iter().enumerate().all(|(i, kid)| {
            let (role_index, _) = kid.role_and_rotation();
            let res = self.exp.contains(&role_index);
            if !res {
                doc.report().invalid_value(
                    "kid",
                    role_index.to_string().as_str(),
                    format!("{:?}", self.exp).as_str(),
                    format!(
                        "Invalid Catalyst Signed Document signature at position [{i}] `kid` Catalyst Role value"
                    )
                    .as_str(),
                );
            }
            res
        });
        if !contains_exp_role {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::{
        catalyst_id::CatalystId,
        uuid::{UuidV4, UuidV7},
    };
    use ed25519_dalek::ed25519::signature::Signer;

    use super::*;
    use crate::{Builder, ContentType};

    #[tokio::test]
    async fn signature_kid_rule_test() {
        let mut rule = SignatureKidRule {
            exp: &[RoleIndex::ROLE_0],
        };

        let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let pk = sk.verifying_key();
        let kid = CatalystId::new("cardano", None, pk).with_role(RoleIndex::ROLE_0);

        let doc = Builder::new()
            .with_decoded_content(serde_json::to_vec(&serde_json::Value::Null).unwrap())
            .with_json_metadata(serde_json::json!({
                "type": UuidV4::new().to_string(),
                "id": UuidV7::new().to_string(),
                "ver": UuidV7::new().to_string(),
                "content-type": ContentType::Json.to_string(),
            }))
            .unwrap()
            .add_signature(|m| sk.sign(&m).to_vec(), &kid)
            .unwrap()
            .build();

        assert!(rule.check(&doc).await.unwrap());

        rule.exp = &[RoleIndex::PROPOSER];
        assert!(!rule.check(&doc).await.unwrap());
    }
}
