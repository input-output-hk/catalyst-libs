//! Cardano RBAC state traits, which are used during different stateful validation
//! procedures.

use std::future::Future;

use cardano_blockchain_types::StakeAddress;
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

    /// Returns a current valid RBAC chain Catalyst ID corresponding to the given stake
    /// address.
    fn chain_catalyst_id_from_stake_address(
        &self,
        address: &StakeAddress,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;

    /// Returns a corresponding to the RBAC chain's Catalyst ID corresponding by the given
    /// public key.
    fn chain_catalyst_id_from_public_key(
        &self,
        key: &VerifyingKey,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;

    /// Update the chain by "taking" the given `StakeAddress` for the corresponding
    /// RBAC chain's by the given `CatalystId`.
    fn take_stake_address_from_chain(
        &mut self,
        id: &CatalystId,
        address: &StakeAddress,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
