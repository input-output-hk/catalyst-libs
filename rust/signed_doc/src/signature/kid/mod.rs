//! COSE Signature Protected Header `kid`.
mod authority;
mod key_version;
mod role;
mod role0_pk;

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use authority::Authority;
use key_version::KeyVersion;
use role::Role;
use role0_pk::Role0PublicKey;

/// Catalyst Signed Document Key ID
///
/// Key ID associated with a `COSE` Signature that is structured as a Universal Resource
/// Identifier (`URI`).
#[derive(Debug, Clone)]
pub struct Kid {
    /// URI Authority
    authority: Authority,
    /// Role0 Public Key.
    role0_public_key: Role0PublicKey,
    /// User Role specified for the current document.
    role: Role,
    /// Role0 Public Key Version
    key_version: KeyVersion,
}

impl Kid {
    /// URI scheme for Catalyst
    const URI_SCHEME_PREFIX: &str = "catalyst_kid://";
}

impl FromStr for Kid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let Some(uri) = s.strip_prefix(Self::URI_SCHEME_PREFIX) else {
            anyhow::bail!("Key ID scheme must be '{}': {s}", Self::URI_SCHEME_PREFIX);
        };

        let Some((authority_str, key_role_version)) = uri.split_once('/') else {
            anyhow::bail!("Key ID must have an authority: {uri}");
        };

        let authority = Authority::from_str(authority_str)
            .map_err(|e| anyhow::anyhow!("Invalid Authority: {authority_str}. {e}"))?;

        let Some((role0_key_str, role_version)) = key_role_version.split_once('/') else {
            anyhow::bail!("Expected Key ID have an Role0 Key set: {key_role_version}");
        };

        let role0_public_key = Role0PublicKey::from_str(role0_key_str)
            .map_err(|e| anyhow::anyhow!("Invalid Role0 Public Key: {role0_key_str}. {e}"))?;

        let Some((role_str, key_version_str)) = role_version.split_once('/') else {
            anyhow::bail!("Expected Key ID have a role set");
        };

        let role = Role::from_str(role_str)
            .map_err(|e| anyhow::anyhow!("Invalid Role: {role_str}. {e}"))?;

        let key_version: KeyVersion = u16::from_str(key_version_str)
            .map_err(|e| anyhow::anyhow!("Invalid Key Version: {key_version_str}. {e}"))?
            .into();

        Ok(Kid {
            authority,
            role0_public_key,
            role,
            key_version,
        })
    }
}

impl Display for Kid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}{}/{}/{}/{}",
            Self::URI_SCHEME_PREFIX,
            self.authority,
            self.role0_public_key,
            self.role,
            self.key_version,
        )
    }
}

impl TryFrom<&[u8]> for Kid {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let kid_str = String::from_utf8_lossy(value);
        Kid::from_str(&kid_str)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Kid;

    const KID_STR: &str = "catalyst_kid://cardano/0x0063ce08eccfdd5c93dd5cc9ca959fe669fd762fa816d70438efa90c0a75288c/3/0";

    #[test]
    fn test_kid_uri_from_str() {
        let kid_str = KID_STR;
        assert!(Kid::from_str(kid_str).is_ok());
    }

    #[test]
    fn test_kid_uri_from_str_and_back() {
        let kid_str = KID_STR;
        let kid = Kid::from_str(kid_str).unwrap();
        assert_eq!(KID_STR, format!("{kid}"));
    }
}
