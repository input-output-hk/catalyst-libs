//! Cardano RBAC state traits, which are used during different statefull validation
//! procedures.

use std::future::Future;

use cardano_blockchain_types::{hashes::TransactionId, StakeAddress};
use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::VerifyingKey;

use crate::registration::cardano::RegistrationChain;

/// RBAC chains state trait
pub trait RBACState {
    /// Returns RBAC chain for the given Catalyst ID.
    fn chain(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<Option<RegistrationChain>>> + Send;

    /// Returns `true` if a RBAC chain with the given Catalyst ID already exists.
    fn is_chain_known(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Returns a current valid RBAC chain corresponding to the given stake
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
