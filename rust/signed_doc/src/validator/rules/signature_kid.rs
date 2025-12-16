//! Catalyst Signed Document COSE signature `kid` (Catalyst Id) role validation

use std::collections::HashSet;

use catalyst_signed_doc_spec::signers::roles::{Roles, UserRole};
use catalyst_types::catalyst_id::role_index::RoleId;

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

///  COSE signature `kid` (Catalyst Id) role validation
#[derive(Debug)]
pub(crate) struct SignatureKidRule {
    /// expected `RoleId` values for the `kid` field.
    /// if empty, document must be signed by admin kid
    allowed_roles: HashSet<RoleId>,
}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for SignatureKidRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl SignatureKidRule {
    /// Generating `SignatureKidRule` from specs
    pub(crate) fn new(spec: &Roles) -> anyhow::Result<Self> {
        anyhow::ensure!(
            spec.user.is_empty() != spec.admin.is_empty(),
            "If 'admin' is not empty 'user' roles cannot been specified'.
            And vice versa, if 'user' is not empty 'admin' roles cannot been specified'"
        );

        let allowed_roles: HashSet<_> = spec
            .user
            .iter()
            .map(|v| {
                match v {
                    UserRole::Registered => RoleId::Role0,
                    UserRole::Proposer => RoleId::Proposer,
                    UserRole::Representative => RoleId::DelegatedRepresentative,
                }
            })
            .collect();

        Ok(Self { allowed_roles })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        let contains_exp_role = doc.authors().iter().enumerate().all(|(i, kid)| {
            if self.allowed_roles.is_empty() {
                let res = kid.is_admin();
                if !res {
                    doc.report().invalid_value(
                        "kid",
                        &kid.to_string(),
                        "Catalyst id must be in admin URI type.",
                        format!(
                            "Invalid Catalyst Signed Document signature at position [{i}] `kid` Catalyst Role value"
                        )
                        .as_str(),
                    );
                }
                res
            } else {
                let (role_index, _) = kid.role_and_rotation();
                let res = self.allowed_roles.contains(&role_index);
                if !res {
                    doc.report().invalid_value(
                        "kid",
                        role_index.to_string().as_str(),
                        format!("{:?}", self.allowed_roles).as_str(),
                        format!(
                            "Invalid Catalyst Signed Document signature at position [{i}] `kid` Catalyst Role value"
                        )
                        .as_str(),
                    );
                }
                res
            }
        });
        if !contains_exp_role {
            return false;
        }

        true
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
    use crate::{ContentType, builder::tests::Builder, metadata::SupportedField};

    #[test]
    fn signature_kid_rule_test() {
        let mut rule = SignatureKidRule {
            allowed_roles: [RoleId::Role0, RoleId::DelegatedRepresentative]
                .into_iter()
                .collect(),
        };

        let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let pk = sk.verifying_key();
        let kid = CatalystId::new("cardano", None, pk).with_role(RoleId::Role0);

        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .with_metadata_field(SupportedField::ContentType(ContentType::Json))
            .with_content(vec![1, 2, 3])
            .add_signature(|m| sk.sign(&m).to_vec(), kid)
            .unwrap()
            .build();

        assert!(rule.check_inner(&doc));

        rule.allowed_roles = [RoleId::Proposer].into_iter().collect();
        assert!(!rule.check_inner(&doc));
    }
}
