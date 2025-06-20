//! Catalyst Signed Document COSE signature `kid` (Catalyst Id) role validation

use catalyst_types::catalyst_id::role_index::RoleId;

use crate::CatalystSignedDocument;

///  COSE signature `kid` (Catalyst Id) role validation
pub(crate) struct SignatureKidRule {
    /// expected `RoleId` values for the `kid` field
    pub(crate) exp: &'static [RoleId],
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
    use crate::{metadata::SupportedField, Builder, ContentType};

    #[tokio::test]
    async fn signature_kid_rule_test() {
        let mut rule = SignatureKidRule {
            exp: &[RoleId::Role0, RoleId::DelegatedRepresentative],
        };

        let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let pk = sk.verifying_key();
        let kid = CatalystId::new("cardano", None, pk).with_role(RoleId::Role0);

        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .with_metadata_field(SupportedField::ContentType(ContentType::Json))
            .with_decoded_content(serde_json::to_vec(&serde_json::Value::Null).unwrap())
            .unwrap()
            .add_signature(|m| sk.sign(&m).to_vec(), kid)
            .unwrap()
            .build();

        assert!(rule.check(&doc).await.unwrap());

        rule.exp = &[RoleId::Proposer];
        assert!(!rule.check(&doc).await.unwrap());
    }
}
