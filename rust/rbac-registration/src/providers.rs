//! Providers traits, which are used during different validation procedures.

use std::future::Future;

use cardano_blockchain_types::{hashes::TransactionId, StakeAddress};
use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::VerifyingKey;

use crate::registration::cardano::RegistrationChain;

/// `RegistrationChain` Provider trait
pub trait RbacRegistrationProvider {
    /// Returns registration chain
    /// for the given Catalyst ID.
    fn chain(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<Option<RegistrationChain>>> + Send;

    /// Returns `true` if a chain with the given Catalyst ID already exists.
    ///
    /// This function behaves in the same way as `latest_rbac_chain(...).is_some()` but
    /// the implementation is more optimized because we don't need to build the whole
    /// chain.
    fn is_chain_known(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Returns a current valid registration chain corresponding to the given stake
    /// address.
    fn chain_from_stake_address(
        &self,
        address: &StakeAddress,
    ) -> impl Future<Output = anyhow::Result<Option<RegistrationChain>>> + Send;

    /// Returns a Catalyst ID corresponding to the given public key.
    fn catalyst_id_from_public_key(
        &self,
        key: &VerifyingKey,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;

    /// Returns a Catalyst ID corresponding to the given transaction hash.
    fn catalyst_id_from_txn_id(
        &self,
        txn_id: &TransactionId,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;
}
