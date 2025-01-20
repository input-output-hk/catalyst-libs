//! COSE Signature Protected Header `kid`.

// cspell: words userinfo rngs Fftx csprng

mod errors;
mod key_rotation;
mod role_index;

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Duration, Utc};
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

/// Catalyst ID
///
/// Identity of Catalyst Registration.
/// Optionally also identifies a specific Signed Document Key
#[derive(Debug, Clone, PartialEq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct IdUri {
    /// Username
    username: Option<String>,
    /// Nonce (like the password in http basic auth, but NOT a password, just a nonce)
    nonce: Option<DateTime<Utc>>,
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
    /// Indicates whether this key is an encryption key.
    /// - `true`: The key is used for encryption.
    /// - `false`: The key is used for signing (signature key).
    encryption: bool,
    /// Indicates if this is an `id` type, or a `uri` type.
    /// Used by the serialization functions.
    /// `true` = format as an `Id`
    /// `false` = format as a `Uri`
    id: bool,
}

impl IdUri {
    /// Encryption Key Identifier Fragment
    const ENCRYPTION_FRAGMENT: &EStr<Fragment> = EStr::new_or_panic("encrypt");
    /// Maximum allowable Nonce Value
    /// * Monday, January 1, 2125 12:00:00 AM
    const MAX_NONCE: i64 = 4_891_363_200;
    /// Minimum allowable Nonce Value
    /// * Wednesday, January 1, 2025 12:00:00 AM
    const MIN_NONCE: i64 = 1_735_689_600;
    /// URI scheme for Catalyst
    const SCHEME: &Scheme = Scheme::new_or_panic("id.catalyst");

    /// Get the cosmetic username from the URI.
    #[must_use]
    pub fn username(&self) -> Option<String> {
        self.username.clone()
    }

    /// Get the nonce from the URI.
    #[must_use]
    pub fn nonce(&self) -> Option<DateTime<Utc>> {
        self.nonce
    }

    /// Get the network the `IdUri` is referencing the registration to.
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

    /// Create a new `IdUri` for a Signing Key
    #[must_use]
    pub fn new(network: &str, subnet: Option<&str>, role0_pk: VerifyingKey) -> Self {
        Self {
            username: None, // Default to Not set, use `with_username` if required.
            nonce: None,    // Default to Not set, use `with_nonce` if required.
            network: network.to_string(),
            subnet: subnet.map(str::to_string),
            role0_pk,
            role: RoleIndex::default(), // Defaulted, use `with_role()` to change it.
            rotation: KeyRotation::default(), // Defaulted, use `with_rotation()` to change it.
            encryption: false,          // Defaulted, use `with_encryption()` to change it.
            id: false,                  // Default to `URI` formatted.
        }
    }

    /// The `IdUri` is formatted as a URI.
    #[must_use]
    pub fn as_uri(self) -> Self {
        Self { id: false, ..self }
    }

    /// The `IdUri` is formatted as a id.
    #[must_use]
    pub fn as_id(self) -> Self {
        Self { id: true, ..self }
    }

    /// Was `IdUri` formatted as a id when it was parsed.
    #[must_use]
    pub fn is_id(self) -> bool {
        self.id
    }

    /// Add or change the username in a Catalyst ID URI.
    #[must_use]
    pub fn with_username(self, name: &str) -> Self {
        Self {
            username: Some(name.to_string()),
            ..self
        }
    }

    /// Add or change the username in a Catalyst ID URI.
    #[must_use]
    pub fn without_username(self) -> Self {
        Self {
            username: None,
            ..self
        }
    }

    /// Add or change the nonce to a specific value in a Catalyst ID URI.
    #[must_use]
    pub fn with_specific_nonce(self, nonce: DateTime<Utc>) -> Self {
        Self {
            nonce: Some(nonce),
            ..self
        }
    }

    /// Add or change the nonce in a Catalyst ID URI.
    #[must_use]
    pub fn with_nonce(self) -> Self {
        self.with_specific_nonce(Utc::now())
    }

    /// Set that there is no Nonce in the ID or URI
    #[must_use]
    pub fn without_nonce(self) -> Self {
        Self {
            nonce: None,
            ..self
        }
    }

    /// Create a new `IdUri` for an Encryption Key
    #[must_use]
    pub fn with_encryption(self) -> Self {
        Self {
            encryption: true,
            ..self
        }
    }

    /// Set that the ID is not for encryption
    #[must_use]
    pub fn without_encryption(self) -> Self {
        Self {
            encryption: false,
            ..self
        }
    }

    /// Set the role explicitly
    #[must_use]
    pub fn with_role(self, role: RoleIndex) -> Self {
        Self { role, ..self }
    }

    /// Set the rotation explicitly
    #[must_use]
    pub fn with_rotation(self, rotation: KeyRotation) -> Self {
        Self { rotation, ..self }
    }

    /// Check if the URI has a nonce that falls within the defined boundary around `now()`
    ///
    /// This function checks whether the nonce (if present) is within the specified time
    /// range relative to the current system time (`now`). The range is determined by
    /// adding and subtracting the given durations (`past` and `future`) from the current
    /// time. If a URI does not have a defined nonce, this function will always return
    /// `false`.
    ///
    /// # Arguments
    ///
    /// * `self`: A reference to the URI object that contains the potential nonce.
    /// * `past`: The duration by which we look back in time from the current moment
    ///   (`now`). (Positive Duration)
    /// * `future`: The duration by which we look forward in time from the current moment
    ///   (`now`). (Positive Duration)
    ///
    /// # Returns
    ///
    /// A boolean value:
    /// - `true` if the nonce is within the specified range relative to `now()`.
    /// - `false` if there is no nonce defined or if the nonce falls outside the specified
    ///   range.
    ///
    /// If the URI does not have a nonce defined, this function returns `false`
    /// immediately because it cannot perform the check without a valid nonce present.
    /// This behavior ensures that the absence of a nonce will fail any required range
    /// checks when such checks are expected according to the function's contract.
    ///
    /// # Examples
    ///
    /// ```
    /// use catalyst_types::id_uri::IdUri;
    /// use chrono::{DateTime, Duration, Utc};
    /// let uri = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<IdUri>()
    ///     .unwrap()
    ///     .with_nonce();
    /// // true, within range
    /// assert!(uri.is_nonce_in_range(chrono::Duration::hours(1), chrono::Duration::minutes(5)));
    ///
    /// // Change the nonce to be 1970/1/1 00:00:00
    /// let uri = uri.with_specific_nonce(DateTime::<Utc>::MIN_UTC);
    /// // false, outside range
    /// assert!(!uri.is_nonce_in_range(chrono::Duration::hours(1), chrono::Duration::minutes(5)));
    /// ```
    #[must_use]
    pub fn is_nonce_in_range(&self, past: Duration, future: Duration) -> bool {
        if let Some(nonce) = self.nonce {
            let now = Utc::now();
            let start_time = now - past;
            let end_time = now + future;
            (start_time..=end_time).contains(&nonce)
        } else {
            // No nonce defined, so we say that this fails.
            // Prevents an absent Nonce from passing range checks when its required.
            false
        }
    }

    /// Convert the `IdUri` to its shortest form.
    /// This is an ID without any role/rotation information, no scheme, no username or
    /// nonce.
    /// This is used to get the most generalized form of a Catalyst ID.
    #[must_use]
    pub fn as_short_id(&self) -> Self {
        self.clone()
            .with_role(RoleIndex::default())
            .with_rotation(KeyRotation::default())
            .without_username()
            .without_nonce()
            .without_encryption()
            .as_id()
    }
}

impl FromStr for IdUri {
    type Err = errors::IdUriError;

    /// This will parse a URI or a RAW ID.  
    /// The only difference between them is a URI has the scheme, a raw ID does not.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Did we serialize an ID?
        let mut id = false;

        // Check if we have a scheme, and if not default it to the catalyst ID scheme.
        let raw_uri = {
            if s.contains("://") {
                s.to_owned()
            } else {
                id = true;
                // It might be a RAW ID, so try and parse with the correct scheme.
                format!("{}://{}", IdUri::SCHEME, s)
            }
        };

        let uri = Uri::parse(raw_uri)?;

        // Check if its the correct scheme.
        if uri.scheme() != IdUri::SCHEME {
            return Err(errors::IdUriError::InvalidScheme);
        }

        // Decode the network and subnet
        let auth = uri
            .authority()
            .ok_or(errors::IdUriError::NoDefinedNetwork)?;
        let (subnet, network) = {
            let host = auth.host();
            if let Some((subnet, host)) = host.rsplit_once('.') {
                (Some(subnet), host)
            } else {
                (None, host)
            }
        };

        let (username, nonce) = {
            if let Some(userinfo) = auth.userinfo() {
                if let Some((username, nonce)) = userinfo.split_once(':') {
                    let username = username.decode().into_string_lossy().to_string();
                    let nonce_str = nonce.decode().into_string_lossy().to_string();

                    let nonce_val: i64 = nonce_str
                        .parse()
                        .map_err(|_| errors::IdUriError::InvalidNonce)?;
                    if !(IdUri::MIN_NONCE..=IdUri::MAX_NONCE).contains(&nonce_val) {
                        return Err(errors::IdUriError::InvalidNonce);
                    }

                    let nonce = DateTime::<Utc>::from_timestamp(nonce_val, 0);

                    (Some(username), nonce)
                } else {
                    let username = userinfo.decode().into_string_lossy().to_string();
                    (Some(username), None)
                }
            } else {
                (None, None)
            }
        };

        let path: Vec<&EStr<Path>> = uri.path().split('/').collect();

        // Can ONLY have 3 path components, no more and no less
        // Less than 3 handled by errors below (4 because of leading `/` in path).
        if path.len() > 4 {
            return Err(errors::IdUriError::InvalidPath);
        };

        // Decode and validate the Role0 Public key from the path
        let encoded_role0_key = path.get(1).ok_or(errors::IdUriError::InvalidRole0Key)?;
        let decoded_role0_key =
            base64_url::decode(encoded_role0_key.decode().into_string_lossy().as_ref())?;
        let role0_pk = crate::conversion::vkey_from_bytes(&decoded_role0_key)
            .or(Err(errors::IdUriError::InvalidRole0Key))?;

        // Decode and validate the Role Index from the path.
        let role_index: RoleIndex = {
            if let Some(encoded_role_index) = path.get(2) {
                let decoded_role_index = encoded_role_index.decode().into_string_lossy();
                decoded_role_index.parse::<RoleIndex>()?
            } else {
                RoleIndex::default()
            }
        };

        // Decode and validate the Rotation Value from the path.
        let rotation: KeyRotation = {
            if let Some(encoded_rotation) = path.get(3) {
                let decoded_rotation = encoded_rotation.decode().into_string_lossy();
                decoded_rotation.parse::<KeyRotation>()?
            } else {
                KeyRotation::default()
            }
        };

        let cat_id = {
            let mut cat_id = Self::new(network, subnet, role0_pk)
                .with_role(role_index)
                .with_rotation(rotation);
            
            if uri.has_fragment() {
                if uri.fragment() == Some(Self::ENCRYPTION_FRAGMENT) {
                    cat_id = cat_id.with_encryption();
                } else {
                    return Err(errors::IdUriError::InvalidEncryptionKeyFragment);
                }
            }

            if let Some(username) = username {
                cat_id = cat_id.with_username(&username);
            }

            if let Some(nonce) = nonce {
                cat_id = cat_id.with_specific_nonce(nonce);
            }

            // Default to URI, so only set it as an ID if its not a URI.
            if id {
                cat_id = cat_id.as_id();
            }

            cat_id
        };

        Ok(cat_id)
    }
}

impl Display for IdUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if !self.id {
            write!(f, "{}://", Self::SCHEME.as_str())?;
        }

        let mut needs_at = false;
        if let Some(username) = &self.username {
            write!(f, "{username}")?;
            needs_at = true;
        }

        if let Some(nonce) = self.nonce {
            let timestamp = nonce.timestamp();
            write!(f, ":{timestamp}")?;
            needs_at = true;
        }

        // If we had a username OR a nonce, then we need an `@` to separate from the hostname.
        if needs_at {
            write!(f, "@")?;
        }

        if let Some(subnet) = &self.subnet {
            write!(f, "{subnet}.")?;
        }
        write!(
            f,
            "{}/{}",
            self.network,
            base64_url::encode(self.role0_pk.as_bytes()),
        )?;

        // Role and Rotation are only serialized if its NOT and ID or they are not the defaults.
        if !self.role.is_default() || !self.rotation.is_default() || !self.id {
            write!(f, "/{}", self.role)?;
            if !self.rotation.is_default() || !self.id {
                write!(f, "/{}", self.rotation)?;
            }
        }

        if self.encryption {
            write!(f, "#{}", Self::ENCRYPTION_FRAGMENT)?;
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for IdUri {
    type Error = errors::IdUriError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let kid_str = String::from_utf8_lossy(value);
        IdUri::from_str(&kid_str)
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    use super::IdUri;

    const ID_URI_TEST_VECTOR: [&str; 9] = [
        "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "user@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "user:1735689600@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        ":1735689600@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3",
        "id.catalyst://preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt",
        "id.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1",
        "id.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/1#encrypt",
    ];

    #[test]
    /// Tests that deserialization and re-serialization round trip correctly
    fn test_id_uri_from_str() {
        for id_string in ID_URI_TEST_VECTOR {
            let id = id_string.parse::<IdUri>().unwrap();
            assert_eq!(format!("{id}"), id_string);
        }
    }

    #[test]
    /// Tests that a short form of a long ID is the same as a short deserialized ID
    fn test_short_id() {
        let test_uri = "id.catalyst://user:1735689600@preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt";
        let expected_id = "preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE";

        let uri_id = test_uri.parse::<IdUri>().unwrap();
        let short_id = expected_id.parse::<IdUri>().unwrap();

        assert_eq!(uri_id.as_short_id(), short_id);
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
