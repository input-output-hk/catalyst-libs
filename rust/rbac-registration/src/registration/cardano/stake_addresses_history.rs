//! An information about stake address used in a RBAC registration chain.

use std::{cmp::Ordering, collections::HashMap};

use anyhow::anyhow;
use cardano_blockchain_types::{Slot, StakeAddress};

#[derive(Debug, Clone)]
pub struct StakeAddressesHistory {
    addresses: HashMap<StakeAddress, Vec<StakeAddressRange>>,
}

/// A half-open range of slots indicating when a stake address was active for some chain.
///
/// Note that ordering for this type is implemented in the following way:
/// - First the ranges are compared by the `active_from` value.
/// - If both ranges have same `active_from` value, than the one with `inactive_from`
///   equal to `None` is considered greater.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeAddressRange {
    /// A slot number when the registration chain started to use the stake address.
    active_from: Slot,
    /// A slot number when the registration chain stopped to use the stake address.
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

    /// Returns a list of addresses sorted by slots.
    pub fn sorted() -> Vec<(StakeAddress, StakeAddressRange)> {
        todo!()
    }
}

impl PartialOrd for StakeAddressRange {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        match self.active_from.partial_cmp(&other.active_from) {
            Some(Ordering::Equal) => {
                match (self.inactive_from, other.inactive_from) {
                    (None, None) => Some(Ordering::Equal),
                    (None, Some(_)) => Some(Ordering::Greater),
                    (Some(_), None) => Some(Ordering::Less),
                    (Some(a), Some(b)) => a.partial_cmp(&b),
                }
            },
            val => val,
        }
    }
}

impl Ord for StakeAddressRange {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.partial_cmp(other)
            .expect("StakeAddressRange should form a total order")
    }
}
