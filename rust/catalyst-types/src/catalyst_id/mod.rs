//! Catalyst ID URI.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/rbac_id_uri/catalyst-id-uri/>

// cspell: words userinfo rngs Fftx csprng

pub mod errors;
pub mod key_rotation;
pub mod role_index;

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    str::FromStr,
    sync::Arc,
};

use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::VerifyingKey;
use fluent_uri::{
    Uri,
    component::Scheme,
    encoding::{
        EStr,
        encoder::{Fragment, Path},
    },
};
use key_rotation::KeyRotation;
use role_index::RoleId;

/// Catalyst ID
/// <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/rbac_id_uri/catalyst-id-uri/>
///
/// Identity of Catalyst Registration. Optionally also identifies a specific Signed
/// Document Key.
///
/// `CatalystId` is an immutable data type: all modifying methods create a new instance.
/// Also, this structure uses [`Arc`] internally, so it is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct CatalystId {
    /// An inner data.
    inner: Arc<CatalystIdInner>,
}

/// A Catalyst ID data intended to be wrapper in `Arc`.
#[derive(Debug, Clone)]
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
    /// Catalyst ID type (URI, ID or Admin URI)
    r#type: CatalystIdType,
}

#[derive(Debug, Clone, Default)]
enum CatalystIdType {
    /// format as an `Id`
    Id,
    /// format as a `Uri`
    #[default]
    Uri,
    /// format as a admin `Uri`
    AdminUri,
}

impl CatalystId {
    /// Admin URI scheme for Catalyst
    pub const ADMIN_SCHEME: &Scheme = Scheme::new_or_panic("admin.catalyst");
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
    pub fn new(
        network: &str,
        subnet: Option<&str>,
        role0_pk: VerifyingKey,
    ) -> Self {
        let inner = Arc::new(CatalystIdInner {
            username: None, // Default to Not set, use `with_username` if required.
            nonce: None,    // Default to Not set, use `with_nonce` if required.
            network: network.to_string(),
            subnet: subnet.map(str::to_string),
            role0_pk,
            role: RoleId::default(), // Defaulted, use `with_role()` to change it.
            rotation: KeyRotation::default(), // Defaulted, use `with_rotation()` to change it.
            encryption: false,       // Defaulted, use `with_encryption()` to change it.
            r#type: CatalystIdType::default(), // Default to `URI` formatted.
        });

        Self { inner }
    }

    /// The `CatalystId` is formatted as a URI.
    #[must_use]
    pub fn as_uri(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            r#type: CatalystIdType::Uri,
            ..inner
        });
        Self { inner }
    }

    /// The `CatalystId` is formatted as a id.
    #[must_use]
    pub fn as_id(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            r#type: CatalystIdType::Id,
            ..inner
        });
        Self { inner }
    }

    /// The `CatalystId` is formatted as a admin URI.
    #[must_use]
    pub fn as_admin(self) -> Self {
        let inner = Arc::try_unwrap(self.inner).unwrap_or_else(|v| (*v).clone());
        let inner = Arc::new(CatalystIdInner {
            r#type: CatalystIdType::AdminUri,
            ..inner
        });
        Self { inner }
    }

    /// Was `CatalystId` formatted as an id when it was parsed.
    #[must_use]
    pub fn is_id(&self) -> bool {
        matches!(self.inner.r#type, CatalystIdType::Id)
    }

    /// Is `CatalystId` formatted as an Admin.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        matches!(self.inner.r#type, CatalystIdType::AdminUri)
    }

    /// Was `CatalystId` formatted as an uri when it was parsed.
    #[must_use]
    pub fn is_uri(&self) -> bool {
        matches!(self.inner.r#type, CatalystIdType::Uri)
    }

    /// Add or change the username in a Catalyst ID URI.
    #[must_use]
    pub fn with_username(
        self,
        name: &str,
    ) -> Self {
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
    pub fn with_specific_nonce(
        self,
        nonce: DateTime<Utc>,
    ) -> Self {
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
    /// use catalyst_types::catalyst_id::{CatalystId, role_index::RoleId};
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
    pub fn with_role(
        self,
        role: RoleId,
    ) -> Self {
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
    /// use catalyst_types::catalyst_id::{CatalystId, key_rotation::KeyRotation};
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
    pub fn with_rotation(
        self,
        rotation: KeyRotation,
    ) -> Self {
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
    pub fn is_nonce_in_range(
        &self,
        past: Duration,
        future: Duration,
    ) -> bool {
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
    /// use catalyst_types::catalyst_id::{CatalystId, key_rotation::KeyRotation, role_index::RoleId};
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

    /// Comparisons of `CatalystId` based on original `PartialEq` plus including
    /// `username` and `nonce` fields.
    #[must_use]
    pub fn eq_with_userinfo(
        &self,
        other: &Self,
    ) -> bool {
        self.eq(other) && self.username().eq(&other.username()) && self.nonce().eq(&other.nonce())
    }

    /// Comparisons of `CatalystId` based on `CatalystId::eq_with_userinfo` plus including
    /// `role` and `rotation` fields.
    #[must_use]
    pub fn eq_with_role(
        &self,
        other: &Self,
    ) -> bool {
        self.eq_with_userinfo(other) && self.role_and_rotation().eq(&other.role_and_rotation())
    }
}

impl PartialEq for CatalystIdInner {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.network.eq(&other.network)
            && self.subnet.eq(&other.subnet)
            && self.role0_pk.eq(&other.role0_pk)
    }
}

impl Eq for CatalystIdInner {}

impl Hash for CatalystIdInner {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.network.hash(state);
        self.subnet.hash(state);
        self.role0_pk.hash(state);
    }
}

impl FromStr for CatalystId {
    type Err = errors::CatalystIdError;

    /// This will parse a URI or a RAW ID.\
    /// The only difference between them is a URI has the scheme, a raw ID does not.
    #[allow(clippy::too_many_lines)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (uri, r#type) = {
            if s.contains("://") {
                let uri = Uri::parse(s.to_owned())
                    .map_err(|e| errors::CatalystIdError::InvalidURI(s.to_string(), e))?;
                // Check if its the correct scheme.
                let r#type = if uri.scheme() == Self::SCHEME {
                    CatalystIdType::Uri
                } else if uri.scheme() == Self::ADMIN_SCHEME {
                    CatalystIdType::AdminUri
                } else {
                    return Err(errors::CatalystIdError::InvalidScheme(s.to_string()));
                };
                (uri, r#type)
            } else {
                // It might be a RAW ID, so try and parse with the correct scheme.
                let uri = Uri::parse(format!("{}://{}", Self::SCHEME, s))
                    .map_err(|e| errors::CatalystIdError::InvalidURI(s.to_string(), e))?;
                let r#type = CatalystIdType::Id;
                (uri, r#type)
            }
        };

        // Decode the network and subnet
        let auth = uri
            .authority()
            .ok_or_else(|| errors::CatalystIdError::NoDefinedNetwork(s.to_string()))?;
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
                        .map_err(|_| errors::CatalystIdError::InvalidNonce(s.to_string()))?;
                    if !(CatalystId::MIN_NONCE..=CatalystId::MAX_NONCE).contains(&nonce_val) {
                        return Err(errors::CatalystIdError::InvalidNonce(s.to_string()));
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
            return Err(errors::CatalystIdError::InvalidPath(s.to_string()));
        }

        // Decode and validate the Role0 Public key from the path
        let encoded_role0_key = path
            .get(1)
            .ok_or_else(|| errors::CatalystIdError::InvalidRole0Key(s.to_string()))?;
        let decoded_role0_key =
            base64_url::decode(encoded_role0_key.decode().into_string_lossy().as_ref())
                .map_err(|e| errors::CatalystIdError::InvalidRole0KeyEncoding(s.to_string(), e))?;
        let role0_pk = crate::conversion::vkey_from_bytes(&decoded_role0_key)
            .or(Err(errors::CatalystIdError::InvalidRole0Key(s.to_string())))?;

        // Decode and validate the Role Index from the path.
        let role_index: RoleId = {
            if let Some(encoded_role_index) = path.get(2) {
                let decoded_role_index = encoded_role_index.decode().into_string_lossy();
                decoded_role_index
                    .parse::<RoleId>()
                    .map_err(|e| errors::CatalystIdError::InvalidRoleId(s.to_string(), e))?
            } else {
                RoleId::default()
            }
        };

        // Decode and validate the Rotation Value from the path.
        let rotation: KeyRotation = {
            if let Some(encoded_rotation) = path.get(3) {
                let decoded_rotation = encoded_rotation.decode().into_string_lossy();
                decoded_rotation
                    .parse::<KeyRotation>()
                    .map_err(|e| errors::CatalystIdError::InvalidRotationValue(s.to_string(), e))?
            } else {
                KeyRotation::default()
            }
        };

        let encryption = match uri.fragment() {
            None => false,
            Some(f) if f == Self::ENCRYPTION_FRAGMENT => true,
            Some(_) => {
                return Err(errors::CatalystIdError::InvalidEncryptionKeyFragment(
                    s.to_string(),
                ));
            },
        };

        let inner = CatalystIdInner {
            network: network.to_string(),
            subnet: subnet.map(ToString::to_string),
            role0_pk,
            r#type,
            rotation,
            role: role_index,
            username,
            nonce,
            encryption,
        }
        .into();

        Ok(Self { inner })
    }
}

impl Display for CatalystId {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        match self.inner.r#type {
            CatalystIdType::Uri => write!(f, "{}://", Self::SCHEME.as_str())?,
            CatalystIdType::AdminUri => write!(f, "{}://", Self::ADMIN_SCHEME.as_str())?,
            CatalystIdType::Id => {},
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
        if !self.inner.role.is_default() || !self.inner.rotation.is_default() || !self.is_id() {
            write!(f, "/{}", self.inner.role)?;
            if !self.inner.rotation.is_default() || !self.is_id() {
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
        let kid_str = String::from_utf8(value.to_vec())
            .map_err(|e| errors::CatalystIdError::InvalidTextEncoding(hex::encode(value), e))?;
        CatalystId::from_str(&kid_str)
    }
}

impl From<&CatalystId> for Vec<u8> {
    fn from(value: &CatalystId) -> Self {
        value.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use test_case::test_case;

    use super::CatalystId;

    const CATALYST_ID_TEST_VECTOR: [&str; 13] = [
        "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "user@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "user:1735689600@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        ":1735689600@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
        "id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3",
        "id.catalyst://preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt",
        "id.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1",
        "id.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/1#encrypt",
        // Admin types
        "admin.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3",
        "admin.catalyst://preview.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt",
        "admin.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1",
        "admin.catalyst://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/1#encrypt",
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

        assert_eq!(uri_id.as_short_id().inner, short_id.inner);
    }

    #[test]
    fn catalyst_id_type_test() {
        for id_string in &CATALYST_ID_TEST_VECTOR[0..5] {
            let id = id_string.parse::<CatalystId>().unwrap();
            assert!(id.is_id());
        }
        for id_string in &CATALYST_ID_TEST_VECTOR[5..9] {
            let id = id_string.parse::<CatalystId>().unwrap();
            assert!(id.is_uri());
        }
        for id_string in &CATALYST_ID_TEST_VECTOR[9..13] {
            let id = id_string.parse::<CatalystId>().unwrap();
            assert!(id.is_admin());
        }
    }

    #[test_case(0, 1, true, false, false; "base vs user")]
    #[test_case(0, 2, true, false, false; "base vs user_nonce")]
    #[test_case(0, 3, true, false, false; "base vs nonce")]
    #[test_case(0, 4, true, true, true; "base vs base_duplicate")]
    #[test_case(7, 8, true, true, false; "midnight_0_1 vs midnight_2_1")]
    #[test_case(0, 5, false, false, false; "cardano vs preprod")]
    #[test_case(5, 6, false, false, false; "preprod vs preview")]
    #[test_case(6, 7, false, false, false; "preview vs midnight")]
    #[test_case(1, 2, true, false, false; "user vs user_nonce")]
    #[test_case(2, 3, true, false, false; "user_nonce vs nonce")]
    #[test_case(8, 8, true, true, true; "identical self comparison")]
    #[allow(clippy::indexing_slicing, clippy::similar_names)]
    fn test_all_comparisons(
        idx_a: usize,
        idx_b: usize,
        expected_eq: bool,
        expected_userinfo: bool,
        expected_role: bool,
    ) {
        let id_a = CATALYST_ID_TEST_VECTOR[idx_a]
            .parse::<CatalystId>()
            .unwrap();
        let id_b = CATALYST_ID_TEST_VECTOR[idx_b]
            .parse::<CatalystId>()
            .unwrap();

        assert_eq!(id_a == id_b, expected_eq, "PartialEq failed");
        assert_eq!(
            id_a.eq_with_userinfo(&id_b),
            expected_userinfo,
            "eq_with_userinfo failed"
        );
        assert_eq!(
            id_a.eq_with_role(&id_b),
            expected_role,
            "eq_with_role failed"
        );
    }

    #[ignore = "Test to be fixed and re-enabled"]
    #[test]
    fn gen_pk() {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let vk = signing_key.verifying_key();
        let encoded_vk = base64_url::encode(vk.as_bytes());
        assert_eq!(encoded_vk, "1234");
    }
}
