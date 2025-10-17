//! Utilities for RBAC registrations validation.

use std::collections::HashSet;

use anyhow::{Context, Result};
use cardano_chain_follower::{hashes::TransactionId, StakeAddress};
use catalyst_types::problem_report::ProblemReport;
use ed25519_dalek::VerifyingKey;

use crate::{
    cardano::cip509::{Cip0134UriSet, Cip509},
    providers::{RbacCacheProvider, RbacRegistrationProvider},
    registration::cardano::{
        validation_result::{RbacValidationError, RbacValidationResult, RbacValidationSuccess},
        RegistrationChain,
    },
};

/// Tries to update an existing RBAC chain.
async fn update_chain<Provider>(
    reg: Cip509,
    previous_txn: TransactionId,
    is_persistent: bool,
    provider: &Provider,
) -> RbacValidationResult
where
    Provider: RbacRegistrationProvider + RbacCacheProvider,
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
    let new_chain = chain.update(reg).ok_or_else(|| {
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

    if is_persistent {
        provider.cache_persistent_rbac_chain(catalyst_id.clone(), new_chain);
    }

    Ok(RbacValidationSuccess {
        catalyst_id,
        stake_addresses,
        public_keys,
        // Only new chains can take ownership of stake addresses of existing chains, so in this case
        // other chains aren't affected.
        modified_chains: Vec::new(),
        purpose,
    })
}

/// Checks that a new registration doesn't contain a signing key that was used by any
/// other chain. Returns a list of public keys in the registration.
async fn validate_public_keys<Provider>(
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
fn cip509_stake_addresses(cip509: &Cip509) -> HashSet<StakeAddress> {
    cip509
        .certificate_uris()
        .map(Cip0134UriSet::stake_addresses)
        .unwrap_or_default()
}
