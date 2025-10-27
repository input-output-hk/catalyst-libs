//! Utilities for RBAC registrations validation.

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use cardano_blockchain_types::StakeAddress;
use catalyst_types::{
    catalyst_id::{role_index::RoleId, CatalystId},
    problem_report::ProblemReport,
    uuid::UuidV4,
};

use crate::{
    cardano::cip509::Cip509, providers::RbacRegistrationProvider,
    registration::cardano::RegistrationChain,
};

/// An error returned from the `validate_rbac_registration` method.
#[allow(clippy::large_enum_variant)]
pub enum RbacValidationError {
    /// A registration is invalid (`report.is_problematic()` returns `true`).
    ///
    /// This variant is inserted to the `rbac_invalid_registration` table.
    InvalidRegistration {
        /// A Catalyst ID.
        catalyst_id: CatalystId,
        /// A registration purpose.
        purpose: Option<UuidV4>,
        /// A problem report.
        report: ProblemReport,
    },
    /// Unable to determine a Catalyst ID of the registration.
    ///
    /// This can happen if a previous transaction ID in the registration is incorrect.
    UnknownCatalystId,
    /// A "fatal" error occurred during validation.
    ///
    /// This means that the validation wasn't performed properly (usually because of a
    /// database failure) and we cannot process the given registration. This error is
    /// propagated on a higher level, so there will be another attempt to index that
    /// block.
    Fatal(anyhow::Error),
}

impl From<anyhow::Error> for RbacValidationError {
    fn from(e: anyhow::Error) -> Self {
        RbacValidationError::Fatal(e)
    }
}

/// Attempts to initialize a new RBAC registration chain
/// from a given CIP-509 registration, ensuring uniqueness of Catalyst ID, stake
/// addresses, and associated public keys.
///
/// # Returns
/// - `Ok((new_chain, validation_result))` if the chain was successfully initialized and
///   validated.
///
/// # Errors
/// - [`RbacValidationError::UnknownCatalystId`]: if `reg` lacks a valid Catalyst ID.
/// - [`RbacValidationError::InvalidRegistration`]: if any functional validation, stake
///   address conflict, or public key duplication occurs.
pub async fn start_new_chain<Provider>(
    reg: Cip509,
    provider: &Provider,
) -> RbacValidationResult
where
    Provider: RbacRegistrationProvider,
{
    let catalyst_id = reg.catalyst_id().map(CatalystId::as_short_id);
    let purpose = reg.purpose();
    let report = reg.report().to_owned();

    // Try to start a new chain.
    let new_chain = RegistrationChain::new(reg.clone()).ok_or_else(|| {
        if let Some(catalyst_id) = catalyst_id {
            RbacValidationError::InvalidRegistration {
                catalyst_id,
                purpose,
                report: report.clone(),
            }
        } else {
            RbacValidationError::UnknownCatalystId
        }
    })?;

    // Verify that a Catalyst ID of this chain is unique.
    let catalyst_id = new_chain.catalyst_id().as_short_id();
    if provider.is_chain_known(catalyst_id.clone()).await? {
        report.functional_validation(
            &format!("{catalyst_id} is already used"),
            "It isn't allowed to use same Catalyst ID (certificate subject public key) in multiple registration chains",
        );
        return Err(RbacValidationError::InvalidRegistration {
            catalyst_id,
            purpose,
            report,
        });
    }

    // Validate stake addresses.
    let new_addresses = new_chain.stake_addresses();
    let mut updated_chains: HashMap<_, HashSet<StakeAddress>> = HashMap::new();
    for address in &new_addresses {
        if let Some(id) = provider.catalyst_id_from_stake_address(address).await? {
            // If an address is used in existing chain then a new chain must have different role 0
            // signing key.
            let previous_chain = provider.chain(id.clone())
                .await?
                .context("{id} is present in 'catalyst_id_for_stake_address', but not in 'rbac_registration'")?;
            if previous_chain.get_latest_signing_pk_for_role(&RoleId::Role0)
                == new_chain.get_latest_signing_pk_for_role(&RoleId::Role0)
            {
                report.functional_validation(
                    &format!("A new registration ({catalyst_id}) uses the same public key as the previous one ({})",
                        previous_chain.catalyst_id().as_short_id()
                    ),
                    "It is only allowed to override the existing chain by using different public key",
                );
            } else {
                // The new root registration "takes" an address(es) from the existing chain, so that
                // chain needs to be updated.
                updated_chains
                    .entry(id)
                    .and_modify(|e| {
                        e.insert(address.clone());
                    })
                    .or_insert([address.clone()].into_iter().collect());
            }
        }
    }

    // Check that new public keys aren't used by other chains.
    let public_keys = new_chain.validate_public_keys(&report, provider).await?;

    if report.is_problematic() {
        return Err(RbacValidationError::InvalidRegistration {
            catalyst_id,
            purpose,
            report,
        });
    }

    Ok(RbacValidationSuccess {
        catalyst_id,
        stake_addresses: new_addresses,
        public_keys,
        modified_chains: updated_chains.into_iter().collect(),
        chain: new_chain,
    })
}
