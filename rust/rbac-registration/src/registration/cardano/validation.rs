//! Utilities for RBAC registrations validation.

use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use cardano_blockchain_types::{hashes::TransactionId, StakeAddress};
use catalyst_types::{
    catalyst_id::{role_index::RoleId, CatalystId},
    problem_report::ProblemReport,
    uuid::UuidV4,
};
use ed25519_dalek::VerifyingKey;

use crate::{
    cardano::cip509::{Cip0134UriSet, Cip509},
    providers::RbacRegistrationProvider,
    registration::cardano::RegistrationChain,
};

/// A return value of the `validate_rbac_registration` method.
pub type RbacValidationResult = Result<RbacValidationSuccess, RbacValidationError>;

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

/// Represents the result yielded by `update_chain` or `start_new_chain` upon successful
/// execution.
pub struct RbacValidationSuccess {
    /// A Catalyst ID of the chain this registration belongs to.
    pub catalyst_id: CatalystId,
    /// A list of stake addresses that were added to the chain.
    pub stake_addresses: HashSet<StakeAddress>,
    /// A list of role public keys used in this registration.
    pub public_keys: HashSet<VerifyingKey>,
    /// A list of updates to other chains containing Catalyst IDs and removed stake
    /// addresses.
    ///
    /// A new RBAC registration can take ownership of stake addresses of other chains.
    pub modified_chains: Vec<(CatalystId, HashSet<StakeAddress>)>,
    /// An updated registration chain.
    pub chain: RegistrationChain,
}

/// Attempts to update an existing RBAC registration chain
/// with a new CIP-509 registration, validating address and key usage consistency.
///
/// # Returns
/// - `Ok((new_chain, validation_result))` if the chain was successfully updated and
///   validated.
///
/// # Errors
/// - Returns [`RbacValidationError::UnknownCatalystId`] if no Catalyst chain is found for
///   `previous_txn`.
/// - Returns [`RbacValidationError::InvalidRegistration`] if address/key duplication or
///   validation inconsistencies are detected.
pub async fn update_chain<Provider>(
    reg: Cip509,
    previous_txn: TransactionId,
    is_persistent: bool,
    provider: &Provider,
) -> RbacValidationResult
where
    Provider: RbacRegistrationProvider,
{
    let purpose = reg.purpose();
    let report = reg.report().to_owned();

    // Find a chain this registration belongs to.
    let Some(catalyst_id) = provider
        .catalyst_id_from_txn_id(previous_txn, is_persistent)
        .await?
    else {
        // We are unable to determine a Catalyst ID, so there is no sense to update the problem
        // report because we would be unable to store this registration anyway.
        return Err(RbacValidationError::UnknownCatalystId);
    };
    let chain = provider.chain(catalyst_id.clone(), is_persistent).await?
        .context("{catalyst_id} is present in 'catalyst_id_for_txn_id' table, but not in 'rbac_registration'")?;

    // Check that addresses from the new registration aren't used in other chains.
    let previous_addresses = chain.stake_addresses();
    let reg_addresses = cip509_stake_addresses(&reg);
    let new_addresses: Vec<_> = reg_addresses.difference(&previous_addresses).collect();
    for address in &new_addresses {
        match provider
            .catalyst_id_from_stake_address(address, is_persistent)
            .await?
        {
            None => {
                // All good: the address wasn't used before.
            },
            Some(_) => {
                report.functional_validation(
                    &format!("{address} stake addresses is already used"),
                    "It isn't allowed to use same stake address in multiple registration chains",
                );
            },
        }
    }

    // Store values before consuming the registration.
    let stake_addresses = cip509_stake_addresses(&reg);

    // Try to add a new registration to the chain.
    let new_chain = chain.update(reg.clone()).ok_or_else(|| {
        RbacValidationError::InvalidRegistration {
            catalyst_id: catalyst_id.clone(),
            purpose,
            report: report.clone(),
        }
    })?;

    // Check that new public keys aren't used by other chains.
    let public_keys = validate_public_keys(&new_chain, is_persistent, &report, provider).await?;

    // Return an error if any issues were recorded in the report.
    if report.is_problematic() {
        return Err(RbacValidationError::InvalidRegistration {
            catalyst_id,
            purpose,
            report,
        });
    }

    Ok(RbacValidationSuccess {
        catalyst_id,
        stake_addresses,
        public_keys,
        // Only new chains can take ownership of stake addresses of existing chains, so in this
        // case other chains aren't affected.
        modified_chains: Vec::new(),
        chain: new_chain,
    })
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
    is_persistent: bool,
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
    if provider
        .is_chain_known(catalyst_id.clone(), is_persistent)
        .await?
    {
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
        if let Some(id) = provider
            .catalyst_id_from_stake_address(address, is_persistent)
            .await?
        {
            // If an address is used in existing chain then a new chain must have different role 0
            // signing key.
            let previous_chain = provider.chain(id.clone(), is_persistent)
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
    let public_keys = validate_public_keys(&new_chain, is_persistent, &report, provider).await?;

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

/// Validates that none of the signing keys in a given RBAC registration chain
/// have been used by any other existing chain, ensuring global key uniqueness
/// across all Catalyst registrations.
///
/// # Returns
/// Returns a [`Result<HashSet<VerifyingKey>>`] containing all unique public keys
/// extracted from the registration chain if validation passes successfully.
///
/// # Errors
/// - Propagates any I/O or provider-level errors encountered while checking key ownership
///   (e.g., database lookup failures).
pub async fn validate_public_keys<Provider>(
    chain: &RegistrationChain,
    is_persistent: bool,
    report: &ProblemReport,
    provider: &Provider,
) -> Result<HashSet<VerifyingKey>>
where
    Provider: RbacRegistrationProvider,
{
    let mut keys = HashSet::new();

    let roles: Vec<_> = chain.role_data_history().keys().collect();
    let catalyst_id = chain.catalyst_id().as_short_id();

    for role in roles {
        if let Some((key, _)) = chain.get_latest_signing_pk_for_role(role) {
            keys.insert(key);
            if let Some(previous) = provider
                .catalyst_id_from_public_key(key, is_persistent)
                .await?
            {
                if previous != catalyst_id {
                    report.functional_validation(
                        &format!("An update to {catalyst_id} registration chain uses the same public key ({key:?}) as {previous} chain"),
                        "It isn't allowed to use role 0 signing (certificate subject public) key in different chains",
                    );
                }
            }
        }
    }

    Ok(keys)
}

/// Returns a set of stake addresses in the given registration.
pub fn cip509_stake_addresses(cip509: &Cip509) -> HashSet<StakeAddress> {
    cip509
        .certificate_uris()
        .map(Cip0134UriSet::stake_addresses)
        .unwrap_or_default()
}
