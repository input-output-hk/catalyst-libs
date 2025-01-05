//! COSE Signature Protected Header `kid` URI Role0 Public Key.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// Role0 Public Key.
#[derive(Debug, Clone)]
pub struct Role0PublicKey([u8; 32]);

impl FromStr for Role0PublicKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(role0_hex) = s.strip_prefix("0x") else {
            anyhow::bail!("Role0 Public Key hex string must start with '0x': {}", s);
        };
        let role0_key = hex::decode(role0_hex)
            .map_err(|e| anyhow::anyhow!("Role0 Public Key is not a valid hex string: {}", e))?;
        if role0_key.len() != 32 {
            anyhow::bail!(
                "Role0 Public Key must have 32 bytes: {role0_hex}, len: {}",
                role0_key.len()
            );
        }
        let role0 = role0_key.try_into().map_err(|e| {
            anyhow::anyhow!(
                "Unable to read Role0 Public Key, this should never happen. Eror: {e:?}"
            )
        })?;
        Ok(Role0PublicKey(role0))
    }
}

impl Display for Role0PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "0x{}", hex::encode(self.0))
    }
}
