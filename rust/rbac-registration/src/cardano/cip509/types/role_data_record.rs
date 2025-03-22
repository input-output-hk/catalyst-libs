//! Role data record where each role data fields are stored.

use std::collections::HashMap;

use pallas::ledger::addresses::ShelleyAddress;

use super::{KeyLocalRef, PointData, PointTxnIdx};

/// Role data record where each field has it own record of its value and its associated
/// point and transaction index. If a field has key rotation, then the vector index is
/// used. Eg. Accessing key rotation 0 can be done by `signing_keys.first()`
#[derive(Debug, Clone)]
pub struct RoleDataRecord {
    /// List of signing key and its associated point + tx index .
    /// The vector index is used to indicate the key rotation.
    signing_keys: Vec<PointData<KeyLocalRef>>,
    /// List of encryption key and its associated point + tx index
    /// The vector index is used to indicate the key rotation.
    encryption_keys: Vec<PointData<KeyLocalRef>>,
    /// List of payment key and its associated point + tx index.
    payment_keys: Vec<PointData<ShelleyAddress>>,
    /// List of extended data and its associated point + tx index.
    extended_data: Vec<PointData<HashMap<u8, Vec<u8>>>>,
}

impl RoleDataRecord {
    /// Create a new empty role data record.
    pub(crate) fn new() -> Self {
        Self {
            signing_keys: Vec::new(),
            encryption_keys: Vec::new(),
            payment_keys: Vec::new(),
            extended_data: Vec::new(),
        }
    }

    /// Add a signing key to the signing key list. If the key already exists, it will
    /// not be added again. The vector index is used to indicate the key rotation.
    pub(crate) fn add_signing_key(&mut self, data: KeyLocalRef, point_tx_idx: &PointTxnIdx) {
        RoleDataRecord::add_key_if_not_exists(&mut self.signing_keys, data, point_tx_idx);
    }

    /// Add an encryption key to the encryption key list. If the key already exists, it
    /// will not be added again. The vector index is used to indicate the key rotation.
    pub(crate) fn add_encryption_key(&mut self, data: KeyLocalRef, point_tx_idx: &PointTxnIdx) {
        RoleDataRecord::add_key_if_not_exists(&mut self.encryption_keys, data, point_tx_idx);
    }

    /// Add a payment key to the payment key list.
    pub(crate) fn add_payment_key(&mut self, data: PointData<ShelleyAddress>) {
        self.payment_keys.push(data);
    }

    /// Add extended data to the extended data list.
    pub(crate) fn add_extended_data(&mut self, data: PointData<HashMap<u8, Vec<u8>>>) {
        self.extended_data.push(data);
    }

    /// Get the list of signing keys associated with this role.
    #[must_use]
    pub fn signing_keys(&self) -> &Vec<PointData<KeyLocalRef>> {
        &self.signing_keys
    }

    /// Get the list of encryption keys associated with this role.
    #[must_use]
    pub fn encryption_keys(&self) -> &Vec<PointData<KeyLocalRef>> {
        &self.encryption_keys
    }

    /// Get the list of payment keys associated with this role.
    #[must_use]
    pub fn payment_keys(&self) -> &Vec<PointData<ShelleyAddress>> {
        &self.payment_keys
    }

    /// Get the list of extended data associated with this role.
    #[must_use]
    pub fn extended_data(&self) -> &Vec<PointData<HashMap<u8, Vec<u8>>>> {
        &self.extended_data
    }

    /// Get the signing key associated with this role and the given key rotation.
    /// The first signing key is at rotation 0, the second at rotation 1, and so on.
    /// Returns `None` if the given key rotation does not exist.
    #[must_use]
    pub fn signing_key_from_rotation(&self, rotation: usize) -> Option<KeyLocalRef> {
        self.signing_keys.get(rotation).map(|pd| pd.data().clone())
    }

    /// Get the encryption key associated with this role and the given key rotation.
    /// The first encryption key is at rotation 0, the second at rotation 1, and so on.
    /// Returns `None` if the given key rotation does not exist.
    #[must_use]
    pub fn encryption_key_from_rotation(&self, rotation: usize) -> Option<KeyLocalRef> {
        self.encryption_keys
            .get(rotation)
            .map(|pd| pd.data().clone())
    }

    /// Helper function to add any key if it doesn't already exist
    fn add_key_if_not_exists<T: PartialEq>(
        keys: &mut Vec<PointData<T>>, data: T, point_tx_idx: &PointTxnIdx,
    ) {
        if !keys.iter().any(|existing_pd| existing_pd.data() == &data) {
            let pd = PointData::new(point_tx_idx.clone(), data);
            keys.push(pd);
        }
    }
}
