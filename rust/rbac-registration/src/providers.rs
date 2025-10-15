//! Providers traits, which are used during different validation procedures.

use std::future::Future;

use cardano_blockchain_types::StakeAddress;
use catalyst_types::catalyst_id::CatalystId;
use ed25519_dalek::VerifyingKey;

use crate::registration::cardano::*;

/// `RegistrationChain` Provider trait
pub trait RbacRegistrationProvider {
    /// Returns either persistent or "latest" (persistent + volatile) registration chain
    /// for the given Catalyst ID.
    fn chain(
        &self,
        id: CatalystId,
        is_persistent: bool,
    ) -> impl Future<Output = anyhow::Result<Option<RegistrationChain>>> + Send;

    /// Returns `true` if a chain with the given Catalyst ID already exists.
    ///
    /// This function behaves in the same way as `latest_rbac_chain(...).is_some()` but
    /// the implementation is more optimized because we don't need to build the whole
    /// chain.
    fn is_chain_known(
        &self,
        id: CatalystId,
        is_persistent: bool,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Returns a Catalyst ID corresponding to the given stake address.
    fn catalyst_id_from_stake_address(
        &self,
        address: &StakeAddress,
        is_persistent: bool,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;

    /// Returns a Catalyst ID corresponding to the given public key.
    fn catalyst_id_from_public_key(
        &self,
        key: VerifyingKey,
        is_persistent: bool,
    ) -> impl Future<Output = anyhow::Result<Option<CatalystId>>> + Send;
}
