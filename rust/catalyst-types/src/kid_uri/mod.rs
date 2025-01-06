//! COSE Signature Protected Header `kid`.
mod authority;
mod errors;
mod key_rotation;
mod role0_pk;
mod role_index;

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use ed25519_dalek::VerifyingKey;
use fluent_uri::{
    component::Scheme,
    encoding::{
        encoder::{Fragment, Path},
        EStr,
    },
    Uri,
};
use key_rotation::KeyRotation;
use role_index::RoleIndex;

/// Catalyst Signed Document Key ID
///
/// Key ID associated with a `COSE` Signature that is structured as a Universal Resource
/// Identifier (`URI`).
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct KidURI {
    /// Network
    network: String,
    /// Sub Network
    subnet: Option<String>,
    /// Role0 Public Key.
    role0_pk: VerifyingKey,
    /// User Role specified for the current document.
    role: RoleIndex,
    /// Role Key Rotation count
    rotation: KeyRotation,
    /// Is this an Encryption Key
    encryption: bool,
}

impl KidURI {
    /// Encryption Key Identifier Fragment
    const ENCRYPTION_FRAGMENT: &EStr<Fragment> = EStr::new_or_panic("encrypt");
    /// URI scheme for Catalyst
    const SCHEME: &Scheme = Scheme::new_or_panic("kid.catalyst-rbac");

    /// Get the network the `KidURI` is referencing the registration to.
    #[must_use]
    pub fn network(&self) -> (String, Option<String>) {
        (self.network.clone(), self.subnet.clone())
    }

    /// Is the key a signature type key.
    #[must_use]
    pub fn is_signature_key(&self) -> bool {
        !self.encryption
    }

    /// Is the key an encryption type key.
    #[must_use]
    pub fn is_encryption_key(&self) -> bool {
        self.encryption
    }

    /// Get the Initial Role 0 Key of the registration
    #[must_use]
    pub fn role0_pk(&self) -> VerifyingKey {
        self.role0_pk
    }

    /// Get the role index and its rotation count
    #[must_use]
    pub fn role_and_rotation(&self) -> (RoleIndex, KeyRotation) {
        (self.role, self.rotation)
    }
}

impl KidURI {
    /// Create a new `KidURI` for a Signing Key
    fn new(
        network: &str, subnet: Option<&str>, role0_pk: VerifyingKey, role: RoleIndex,
        rotation: KeyRotation,
    ) -> Self {
        Self {
            network: network.to_string(),
            subnet: subnet.map(str::to_string),
            role0_pk,
            role,
            rotation,
            encryption: false,
        }
    }

    /// Create a new `KidURI` for an Encryption Key
    fn new_encryption(
        network: &str, subnet: Option<&str>, role0_pk: VerifyingKey, role: RoleIndex,
        rotation: KeyRotation,
    ) -> Self {
        let mut kid = Self::new(network, subnet, role0_pk, role, rotation);
        kid.encryption = true;
        kid
    }
}

impl FromStr for KidURI {
    type Err = errors::KidURIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = Uri::parse(s)?;

        // Check if its the correct scheme.
        if uri.scheme() != KidURI::SCHEME {
            return Err(errors::KidURIError::InvalidScheme);
        }

        // Decode the network and subnet
        let auth = uri
            .authority()
            .ok_or(errors::KidURIError::NoDefinedNetwork)?;
        let network = auth.host();
        let subnet = auth.userinfo().map(std::string::ToString::to_string);

        let path: Vec<&EStr<Path>> = uri.path().split('/').collect();

        // Can ONLY have 3 path components, no more and no less
        // Less than 3 handled by errors below (4 because of leading `/` in path).
        if path.len() > 4 {
            return Err(errors::KidURIError::InvalidPath);
        };

        // Decode and validate the Role0 Public key from the path
        let encoded_role0_key = path.get(1).ok_or(errors::KidURIError::InvalidRole0Key)?;
        let decoded_role0_key =
            base64_url::decode(encoded_role0_key.decode().into_string_lossy().as_ref())?;
        let role0_pk = crate::conversion::vkey_from_bytes(&decoded_role0_key)
            .or(Err(errors::KidURIError::InvalidRole0Key))?;

        // Decode and validate the Role Index from the path.
        let encoded_role_index = path.get(2).ok_or(errors::KidURIError::InvalidRole)?;
        let decoded_role_index = encoded_role_index.decode().into_string_lossy();
        let role_index = decoded_role_index.parse::<RoleIndex>()?;

        // Decode and validate the Rotation Value from the path.
        let encoded_rotation = path.get(3).ok_or(errors::KidURIError::InvalidRotation)?;
        let decoded_rotation = encoded_rotation.decode().into_string_lossy();
        let rotation = decoded_rotation.parse::<KeyRotation>()?;

        let kid = {
            if uri.has_fragment() {
                if uri.fragment() == Some(Self::ENCRYPTION_FRAGMENT) {
                    Self::new_encryption(network, subnet.as_deref(), role0_pk, role_index, rotation)
                } else {
                    return Err(errors::KidURIError::InvalidEncryptionKeyFragment);
                }
            } else {
                Self::new(network, subnet.as_deref(), role0_pk, role_index, rotation)
            }
        };

        Ok(kid)
    }
}

impl Display for KidURI {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}://", Self::SCHEME.as_str())?;
        if let Some(subnet) = &self.subnet {
            write!(f, "{subnet}@")?;
        }
        write!(
            f,
            "{}/{}/{}/{}",
            self.network,
            base64_url::encode(self.role0_pk.as_bytes()),
            self.role,
            self.rotation
        )?;
        if self.encryption {
            write!(f, "#{}", Self::ENCRYPTION_FRAGMENT)?;
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for KidURI {
    type Error = errors::KidURIError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let kid_str = String::from_utf8_lossy(value);
        KidURI::from_str(&kid_str)
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    use super::KidURI;

    const KID_TEST_VECTOR: [&str; 5] = [
        "kid.catalyst-rbac://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/0",
        "kid.catalyst-rbac://preprod@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3",
        "kid.catalyst-rbac://preprod@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt",
        "kid.catalyst-rbac://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1",
        "kid.catalyst-rbac://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/1#encrypt"
    ];

    #[test]
    fn test_kid_uri_from_str() {
        for kid_string in KID_TEST_VECTOR {
            let kid = kid_string.parse::<KidURI>().unwrap();
            assert_eq!(format!("{kid}"), kid_string);
        }
    }

    #[ignore]
    #[test]
    fn gen_pk() {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let vk = signing_key.verifying_key();
        let encoded_vk = base64_url::encode(vk.as_bytes());
        assert_eq!(encoded_vk, "1234");
    }
}