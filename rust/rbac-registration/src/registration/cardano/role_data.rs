//! RBAC role data

use std::collections::HashMap;

use super::cip19_shelley_addr::Cip19ShelleyAddrs;
use crate::cardano::cip509::rbac::role_data::KeyLocalRef;

/// Role data
#[derive(Clone)]
pub struct RoleData {
    /// A signing keys to the data within registration.
    signing_key_ref: Option<KeyLocalRef>,
    /// An encryption keys to the data within registration.
    encryption_ref: Option<KeyLocalRef>,
    /// A payment key (Shelley address) where reward will be distributed to.
    payment_key: Cip19ShelleyAddrs,
    /// Map of role extended data (10-99) to its data
    role_extended_data: HashMap<u8, Vec<u8>>,
}

impl RoleData {
    /// Create an instance of role data.
    pub(crate) fn new(
        signing_key_ref: Option<KeyLocalRef>, encryption_ref: Option<KeyLocalRef>,
        payment_key: Cip19ShelleyAddrs, role_extended_data: HashMap<u8, Vec<u8>>,
    ) -> Self {
        RoleData {
            signing_key_ref,
            encryption_ref,
            payment_key,
            role_extended_data,
        }
    }

    /// Get the reference of signing keys.
    #[must_use]
    pub fn signing_key_ref(&self) -> &Option<KeyLocalRef> {
        &self.signing_key_ref
    }

    /// Get the reference of encryption keys.
    #[must_use]
    pub fn encryption_ref(&self) -> &Option<KeyLocalRef> {
        &self.encryption_ref
    }

    /// Get the payment key.
    #[must_use]
    pub fn payment_key(&self) -> &Cip19ShelleyAddrs {
        &self.payment_key
    }

    /// Get the role extended data.
    #[must_use]
    pub fn role_extended_data(&self) -> &HashMap<u8, Vec<u8>> {
        &self.role_extended_data
    }
}
