//! Cardano RBAC provider trait, which are used during different stateful validation procedures.

use std::future::Future;

use cardano_blockchain_types::StakeAddress;
use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::VerifyingKey;

use crate::registration::cardano::RegistrationChain;

/// RBAC chains provider trait
pub trait RbacChainsProvider: Send + Sync {
    /// Returns RBAC chain for the given Catalyst ID.
    fn chain(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<Option<RegistrationChain>>> + Send;

    /// Returns `true` if a RBAC chain with the given Catalyst ID already exists.
    fn is_chain_known(
        &self,
        id: &CatalystId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send {
        async { self.chain(id).await.map(|v| v.is_some()) }
    }

    /// Returns the Catalyst ID associated with the RBAC chain for the given stake
    /// address.
    fn chain_catalyst_id_from_stake_address(
        &self,
        addr: &StakeAddress,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;

    /// Returns `true` if a provided address already used by any RBAC chain.
    fn is_stake_address_used(
        &self,
        addr: &StakeAddress,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send {
        async {
            self.chain_catalyst_id_from_stake_address(addr)
                .await
                .map(|v| v.is_some())
        }
    }

    /// Returns the Catalyst ID associated with the RBAC chain for the given signing
    /// public key.
    fn chain_catalyst_id_from_signing_public_key(
        &self,
        key: &VerifyingKey,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;
}
