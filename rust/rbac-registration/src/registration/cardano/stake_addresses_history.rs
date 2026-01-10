//! An information about stake address used in a RBAC registration chain.

use std::collections::HashMap;

use anyhow::anyhow;
use cardano_blockchain_types::{Slot, StakeAddress};

#[derive(Debug, Clone)]
pub struct StakeAddressesHistory {
    addresses: HashMap<StakeAddress, Vec<StakeAddressRange>>,
}

#[derive(Debug, Clone)]
pub struct StakeAddressRange {
    active_from: Slot,
    inactive_from: Option<Slot>,
}

impl StakeAddressesHistory {
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
        }
    }

    pub fn record_addresses(
        &mut self,
        addresses: &[StakeAddress],
        slot: Slot,
    ) {
        todo!()
    }

    pub fn remove_address(
        &mut self,
        addresses: &[StakeAddress],
        slot: Slot,
    ) -> anyhow::Result<()> {
        for address in addresses {
            let Some(ranges) = self.addresses.get_mut(address) else {
                return Err(anyhow!(
                    "Unable to record {address} address as removed as it isn't present in history"
                ));
            };

            let Some(range) = ranges.last_mut() else {
                return Err(anyhow!(
                    "Inconsistent state: {address} address is known, but the ranges list is empty"
                ));
            };

            if let Some(inactive_from) = range.inactive_from {
                return Err(anyhow!(
                    "Unable to mark address is inactive ({slot:?}): it is already inactive from {inactive_from:?}"
                ));
            }

            range.inactive_from = Some(slot);
        }

        Ok(())
    }
}
