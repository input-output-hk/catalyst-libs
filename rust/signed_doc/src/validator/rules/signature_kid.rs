//! Catalyst Signed Document COSE signature `kid` (Catalyst Id) role validation

use catalyst_types::id_uri::role_index::RoleIndex;

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
            doc.report().invalid_value(
                "kid",
                role_index.to_string().as_str(),
                format!("{:?}", self.exp).as_str(),
                format!("Invalid Catalyst Signed Document signature {i} `kid` Catalyst Role value")
                    .as_str(),
            );
            self.exp.contains(&role_index)
        });
        if !contains_exp_role {
            return Ok(false);
        }

        Ok(true)
    }
}
