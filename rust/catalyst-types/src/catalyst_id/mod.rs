//! Catalyst ID URI.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/rbac_id_uri/catalyst-id-uri/>

// cspell: words userinfo rngs Fftx csprng

pub mod errors;
pub mod key_rotation;
pub mod role_index;

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    sync::Arc,
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
use role_index::RoleId;

/// Catalyst ID
/// <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/rbac_id_uri/catalyst-id-uri/>
///
/// Identity of Catalyst Registration.
/// Optionally also identifies a specific Signed Document Key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct CatalystId {
    /// An inner data.
    inner: Arc<CatalystIdInner>,
}

/// A Catalyst ID data intended to be wrapper in `Arc`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CatalystIdInner {
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
    role: RoleId,
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

impl CatalystId {
    /// Encryption Key Identifier Fragment
    const ENCRYPTION_FRAGMENT: &EStr<Fragment> = EStr::new_or_panic("encrypt");
    /// Maximum allowable Nonce Value
    /// * Monday, January 1, 2125 12:00:00 AM
    const MAX_NONCE: i64 = 4_891_363_200;
    /// Minimum allowable Nonce Value
    /// * Wednesday, January 1, 2025 12:00:00 AM
    const MIN_NONCE: i64 = 1_735_689_600;
    /// URI scheme for Catalyst
    pub const SCHEME: &Scheme = Scheme::new_or_panic("id.catalyst");

    /// Get the cosmetic username from the URI.
    #[must_use]
    pub fn username(&self) -> Option<String> {
        self.inner.username.clone()
    }

    /// Get the nonce from the URI.
    #[must_use]
    pub fn nonce(&self) -> Option<DateTime<Utc>> {
        self.inner.nonce
    }

    /// Get the network the `CatalystId` is referencing the registration to.
    #[must_use]
    pub fn network(&self) -> (String, Option<String>) {
        (self.inner.network.clone(), self.inner.subnet.clone())
    }

    /// Is the key a signature type key.
    #[must_use]
    pub fn is_signature_key(&self) -> bool {
        !self.inner.encryption
    }

    /// Is the key an encryption type key.
    #[must_use]
    pub fn is_encryption_key(&self) -> bool {
        self.inner.encryption
    }

    /// Get the Initial Role 0 Key of the registration
    #[must_use]
    pub fn role0_pk(&self) -> VerifyingKey {
        self.inner.role0_pk
    }

    /// Get the role index and its rotation count
    #[must_use]
    pub fn role_and_rotation(&self) -> (RoleId, KeyRotation) {
        (self.inner.role, self.inner.rotation)
    }

    /// Create a new `CatalystId` for a Signing Key
    #[must_use]
    pub fn new(network: &str, subnet: Option<&str>, role0_pk: VerifyingKey) -> Self {
        let inner = Arc::new(CatalystIdInner {
            username: None, // Default to Not set, use `with_username` if required.
            nonce: None,    // Default to Not set, use `with_nonce` if required.
            network: network.to_string(),
            subnet: subnet.map(str::to_string),
            role0_pk,
            role: RoleId::default(), // Defaulted, use `with_role()` to change it.
            rotation: KeyRotation::default(), // Defaulted, use `with_rotation()` to change it.
            encryption: false,       // Defaulted, use `with_encryption()` to change it.
            id: false,               // Default to `URI` formatted.
        });

        Self { inner }
    }

    /// The `CatalystId` is formatted as a URI.
    #[must_use]
    pub fn as_uri(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner { id: false, ..inner });
        Self { inner }
    }

    /// The `CatalystId` is formatted as a id.
    #[must_use]
    pub fn as_id(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner { id: true, ..inner });
        Self { inner }
    }

    /// Was `CatalystId` formatted as an id when it was parsed.
    #[must_use]
    pub fn is_id(&self) -> bool {
        self.inner.id
    }

    /// Was `CatalystId` formatted as an uri when it was parsed.
    #[must_use]
    pub fn is_uri(&self) -> bool {
        !self.inner.id
    }

    /// Add or change the username in a Catalyst ID URI.
    #[must_use]
    pub fn with_username(self, name: &str) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            username: Some(name.to_string()),
            ..inner
        });
        Self { inner }
    }

    /// Add or change the username in a Catalyst ID URI.
    #[must_use]
    pub fn without_username(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            username: None,
            ..inner
        });
        Self { inner }
    }

    /// Add or change the nonce (a unique identifier for a data update) to a specific
    /// value in a Catalyst `CatalystId`.
    ///
    /// This method is intended for use with trusted data where the nonce is known and
    /// verified beforehand. It ensures that the provided nonce is within the valid
    /// range, clamping it if necessary between `MIN_NONCE` and `MAX_NONCE`.
    /// Properly generated or trusted nonces will not be altered by this function.
    ///
    /// # Parameters
    /// - `nonce`: A `DateTime` representing the specific nonce value to set in the
    ///   Catalyst `CatalystId`. This should be a valid UTC datetime.
    ///
    /// # Returns
    /// The updated Catalyst `CatalystId` with the specified nonce, if it was within the
    /// allowed range; otherwise, it will be updated with a clamped value of the
    /// nonce.
    ///
    /// # Safety
    /// - **Pre-validation of the nonce is required**: If you are working with untrusted
    ///   data, ensure that the nonce has been pre-validated and take appropriate action
    ///   before calling this function.
    #[must_use]
    pub fn with_specific_nonce(self, nonce: DateTime<Utc>) -> Self {
        let secs = nonce.timestamp();
        let clamped_secs = secs.clamp(Self::MIN_NONCE, Self::MAX_NONCE);

        let nonce = {
            if clamped_secs == secs {
                Some(nonce)
            } else {
                DateTime::<Utc>::from_timestamp(clamped_secs, 0)
            }
        };

        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner { nonce, ..inner });
        Self { inner }
    }

    /// Add or change the nonce in a Catalyst ID URI. The nonce will be set to the current
    /// UTC time when this method is called.
    ///
    /// This function returns a new instance of the type with the nonce field updated to
    /// the current UTC time.
    ///
    /// # Examples
    /// ```rust
    /// use catalyst_types::catalyst_id::CatalystId;
    /// use chrono::Utc;
    ///
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
    ///     .unwrap();
    /// assert!(catalyst_id.nonce().is_none());
    /// let catalyst_id_with_nonce = catalyst_id.with_nonce();
    /// assert!(catalyst_id_with_nonce.nonce().is_some());
    /// ```
    #[must_use]
    pub fn with_nonce(self) -> Self {
        self.with_specific_nonce(Utc::now())
    }

    /// Set that there is no Nonce in the ID or URI
    /// Represents an ID or URI without a Nonce.
    ///
    /// This method creates a new instance of the type, but sets the nonce field to
    /// `None`. The rest of the fields are inherited from the original instance.
    ///
    /// # Examples
    /// ```rust
    /// use catalyst_types::catalyst_id::CatalystId;
    /// use chrono::{DateTime, Duration, Utc};
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
    ///     .unwrap()
    ///     .with_nonce();
    ///
    /// let catalyst_id_without_nonce = catalyst_id.without_nonce();
    /// assert_eq!(catalyst_id_without_nonce.nonce(), None);
    /// ```
    #[must_use]
    pub fn without_nonce(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            nonce: None,
            ..inner
        });
        Self { inner }
    }

    /// Set that the `CatalystId` is used to identify an encryption key.
    ///
    /// This method sets `CatalystId` is identifying an encryption key.
    ///
    /// # Returns
    ///
    /// A new instance of the type with the updated encryption flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use catalyst_types::catalyst_id::CatalystId;
    ///
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
    ///     .unwrap();
    /// assert_eq!(catalyst_id.is_encryption_key(), false);
    ///
    /// let catalyst_id = catalyst_id.with_encryption();
    /// assert_eq!(catalyst_id.is_encryption_key(), true);
    /// ```
    #[must_use]
    pub fn with_encryption(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            encryption: true,
            ..inner
        });
        Self { inner }
    }

    /// Set that the `CatalystId` is not for encryption
    ///
    /// This method sets `CatalystId` is not identifying an encryption key.
    ///
    /// # Returns
    ///
    /// A new instance of the type with the updated encryption flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use catalyst_types::catalyst_id::CatalystId;
    ///
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE#encrypt"
    ///     .parse::<CatalystId>()
    ///     .unwrap();
    /// assert_eq!(catalyst_id.is_encryption_key(), true);
    ///
    /// let catalyst_id = catalyst_id.without_encryption();
    /// assert_eq!(catalyst_id.is_encryption_key(), false);
    /// ```
    #[must_use]
    pub fn without_encryption(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            encryption: false,
            ..inner
        });
        Self { inner }
    }

    /// Set the role explicitly.
    ///
    /// This method sets the role field to the specified value while leaving other
    /// fields unchanged.
    ///
    /// # Parameters
    /// - `role`: The new value for the role field.
    ///
    /// # Returns
    ///
    /// A new instance of the type with the updated role field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use catalyst_types::catalyst_id::{role_index::RoleId, CatalystId};
    ///
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
    ///     .unwrap();
    /// let new_role = RoleId::Proposer;
    /// let catalyst_id_with_role = catalyst_id.with_role(new_role);
    /// let (role, _) = catalyst_id_with_role.role_and_rotation();
    /// assert_eq!(role, new_role);
    /// ```
    #[must_use]
    pub fn with_role(self, role: RoleId) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner { role, ..inner });
        Self { inner }
    }

    /// Set the rotation explicitly.
    ///
    /// This method sets the rotation field to the specified value while leaving other
    /// fields unchanged.
    ///
    /// # Parameters
    /// - `rotation`: The new value for the rotation field.  0 = First Key, 1+ is each
    ///   subsequent rotation.
    ///
    /// # Returns
    /// A new instance of the type with the updated rotation field.
    ///
    /// # Examples
    /// ```rust
    /// use catalyst_types::catalyst_id::{key_rotation::KeyRotation, CatalystId};
    ///
    /// let catalyst_id = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
    ///     .unwrap();
    /// let new_rotation: KeyRotation = 4.into();
    /// let catalyst_id_with_rotation = catalyst_id.with_rotation(new_rotation);
    /// let (_, rotation) = catalyst_id_with_rotation.role_and_rotation();
    /// assert_eq!(rotation, new_rotation);
    /// ```
    #[must_use]
    pub fn with_rotation(self, rotation: KeyRotation) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner { rotation, ..inner });
        Self { inner }
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
    /// use catalyst_types::catalyst_id::CatalystId;
    /// use chrono::{DateTime, Duration, Utc};
    /// let uri = "id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    ///     .parse::<CatalystId>()
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
        if let Some(nonce) = self.nonce() {
            let now = Utc::now();
            let Some(start_time) = now.checked_sub_signed(past) else {
                return false;
            };
            let Some(end_time) = now.checked_add_signed(future) else {
                return false;
            };
            (start_time..=end_time).contains(&nonce)
        } else {
            // No nonce defined, so we say that this fails.
            // Prevents an absent Nonce from passing range checks when its required.
            false
        }
    }

    /// Converts the `CatalystId` to its shortest form.
    /// This method returns a new instance of the type with no role information, no
    /// scheme, no username, no nonce, and no encryption settings. It effectively
    /// strips away all additional metadata to provide a most generalized form of the
    /// Catalyst ID.
    ///
    /// # Returns
    ///
    /// A new `CatalystId` instance representing the shortest form of the current
    /// `CatalystId`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use catalyst_types::catalyst_id::{key_rotation::KeyRotation, role_index::RoleId, CatalystId};
    ///
    /// let catalyst_id =
    ///     "id.catalyst://user:1735689600@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/5"
    ///         .parse::<CatalystId>()
    ///         .unwrap();
    ///
    /// let short_id = catalyst_id.as_short_id();
    /// assert_eq!(
    ///     short_id.role_and_rotation(),
    ///     (RoleId::default(), KeyRotation::default())
    /// );
    /// assert_eq!(short_id.username(), None);
    /// assert_eq!(short_id.nonce(), None);
    /// assert_eq!(short_id.is_encryption_key(), false);
    ///
    /// let short_id_str = format!("{short_id}");
    /// assert_eq!(
    ///     short_id_str,
    ///     "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE"
    /// )
    /// ```
    #[must_use]
    pub fn as_short_id(&self) -> Self {
        self.clone()
            .with_role(RoleId::default())
            .with_rotation(KeyRotation::default())
            .without_username()
            .without_nonce()
            .without_encryption()
            .as_id()
    }
}

impl FromStr for CatalystId {
    type Err = errors::CatalystIdError;

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
                format!("{}://{}", CatalystId::SCHEME, s)
            }
        };

        let uri = Uri::parse(raw_uri)?;

        // Check if its the correct scheme.
        if uri.scheme() != CatalystId::SCHEME {
            return Err(errors::CatalystIdError::InvalidScheme);
        }

        // Decode the network and subnet
        let auth = uri
            .authority()
            .ok_or(errors::CatalystIdError::NoDefinedNetwork)?;
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
                        .map_err(|_| errors::CatalystIdError::InvalidNonce)?;
                    if !(CatalystId::MIN_NONCE..=CatalystId::MAX_NONCE).contains(&nonce_val) {
                        return Err(errors::CatalystIdError::InvalidNonce);
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
            return Err(errors::CatalystIdError::InvalidPath);
        };

        // Decode and validate the Role0 Public key from the path
        let encoded_role0_key = path
            .get(1)
            .ok_or(errors::CatalystIdError::InvalidRole0Key)?;
        let decoded_role0_key =
            base64_url::decode(encoded_role0_key.decode().into_string_lossy().as_ref())?;
        let role0_pk = crate::conversion::vkey_from_bytes(&decoded_role0_key)
            .or(Err(errors::CatalystIdError::InvalidRole0Key))?;

        // Decode and validate the Role Index from the path.
        let role_index: RoleId = {
            if let Some(encoded_role_index) = path.get(2) {
                let decoded_role_index = encoded_role_index.decode().into_string_lossy();
                decoded_role_index.parse::<RoleId>()?
            } else {
                RoleId::default()
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
                    return Err(errors::CatalystIdError::InvalidEncryptionKeyFragment);
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

impl Display for CatalystId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if !self.inner.id {
            write!(f, "{}://", Self::SCHEME.as_str())?;
        }

        let mut needs_at = false;
        if let Some(username) = &self.inner.username {
            write!(f, "{username}")?;
            needs_at = true;
        }

        if let Some(nonce) = self.nonce() {
            let timestamp = nonce.timestamp();
            write!(f, ":{timestamp}")?;
            needs_at = true;
        }

        // If we had a username OR a nonce, then we need an `@` to separate from the hostname.
        if needs_at {
            write!(f, "@")?;
        }

        if let Some(subnet) = &self.inner.subnet {
            write!(f, "{subnet}.")?;
        }
        write!(
            f,
            "{}/{}",
            self.inner.network,
            base64_url::encode(self.role0_pk().as_bytes()),
        )?;

        // Role and Rotation are only serialized if its NOT and ID or they are not the defaults.
        if !self.inner.role.is_default() || !self.inner.rotation.is_default() || !self.inner.id {
            write!(f, "/{}", self.inner.role)?;
            if !self.inner.rotation.is_default() || !self.inner.id {
                write!(f, "/{}", self.inner.rotation)?;
            }
        }

        if self.inner.encryption {
            write!(f, "#{}", Self::ENCRYPTION_FRAGMENT)?;
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for CatalystId {
    type Error = errors::CatalystIdError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let kid_str = String::from_utf8(value.to_vec())?;
        CatalystId::from_str(&kid_str)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    use super::CatalystId;

    const CATALYST_ID_TEST_VECTOR: [&str; 9] = [
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
    fn test_catalyst_id_from_str() {
        for id_string in CATALYST_ID_TEST_VECTOR {
            let username = id_string.split_once('@').map(|s| s.0);
            let (username, nonce) = username
                .and_then(|s| s.split_once(':').map(|(u, n)| (u, Some(n))))
                .or_else(|| username.map(|u| (u, None)))
                .unzip();

            let nonce = nonce.flatten();
            let username = username.map(String::from);
            let nonce = nonce
                .map(|n| n.parse::<i64>().unwrap())
                .map(|n| DateTime::<Utc>::from_timestamp(n, 0).unwrap());

            let encryption =
                id_string.contains(format!("#{}", CatalystId::ENCRYPTION_FRAGMENT).as_str());

            let id = id_string.parse::<CatalystId>().unwrap();

            assert_eq!(format!("{id}"), id_string);
            assert_eq!(username, id.username());
            assert_eq!(nonce, id.nonce());
            assert!(id.is_signature_key() ^ encryption);
            assert!(id.is_encryption_key() ^ !encryption);
        }
    }

    #[test]
    /// Tests that a short form of a long ID is the same as a short deserialized ID
    fn test_short_id() {
        let test_uri = "id.catalyst://user:1735689600@preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt";
        let expected_id = "preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE";

        let uri_id = test_uri.parse::<CatalystId>().unwrap();
        let short_id = expected_id.parse::<CatalystId>().unwrap();

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
