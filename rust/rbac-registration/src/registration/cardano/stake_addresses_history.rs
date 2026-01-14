//! An information about stake address used in a RBAC registration chain.

use std::{cmp::Ordering, collections::HashMap};

use anyhow::anyhow;
use cardano_blockchain_types::{Slot, StakeAddress};

use crate::cardano::cip509::Cip0134UriSet;

#[derive(Debug, Clone)]
pub struct StakeAddressesHistory {
    addresses: HashMap<StakeAddress, Vec<StakeAddressRange>>,
}

/// A half-open range of slots indicating when a stake address was active for some chain.
///
/// Note that ordering for this type is implemented in the following way:
/// - First the ranges are compared by the `active_from` value.
/// - If both ranges have same `active_from` value, then the one with `inactive_from`
///   equal to `None` is considered greater.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeAddressRange {
    /// A slot number when the registration chain started to use the stake address.
    active_from: Slot,
    /// A slot number when the registration chain stopped to use the stake address.
    inactive_from: Option<Slot>,
}

impl StakeAddressesHistory {
    /// Creates a new `StakeAddressesHistory` instance with the given stake addresses set
    /// and slot.
    pub fn new(
        certificate_uris: &Cip0134UriSet,
        slot: Slot,
    ) -> Self {
        // This is called for the first registration in a chain, so all stake addresses are active
        // and there are no removed ones.
        let addresses = certificate_uris
            .active_stake_addresses()
            .into_iter()
            .map(|a| {
                (a, vec![StakeAddressRange {
                    active_from: slot,
                    inactive_from: None,
                }])
            })
            .collect();
        Self { addresses }
    }

    pub fn record_addresses(
        &mut self,
        _addresses: &[StakeAddress],
        _slot: Slot,
    ) {
        // TODO: FIXME:
        todo!()
    }

    /// Marks the given addresses as removed.
    pub fn remove_addresses(
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
    #[must_use]
    pub fn sorted(&self) -> Vec<(StakeAddress, StakeAddressRange)> {
        let mut res: Vec<_> = self
            .addresses
            .iter()
            .flat_map(|(address, ranges)| {
                ranges.iter().cloned().map(|range| (address.clone(), range))
            })
            .collect();
        res.sort_by(|(_, a), (_, b)| a.cmp(b));
        res
    }
}

impl Ord for StakeAddressRange {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        match self.active_from.cmp(&other.active_from) {
            Ordering::Equal => {
                match (self.inactive_from, other.inactive_from) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(_), None) => Ordering::Less,
                    (Some(a), Some(b)) => a.cmp(&b),
                }
            },
            val => val,
        }
    }
}

impl PartialOrd for StakeAddressRange {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use cardano_blockchain_types::Network;

    use super::*;

    #[test]
    fn sorted() {
        let stake_1 = {
            let hash = "276fd18711931e2c0e21430192dbeac0e458093cd9d1fcd7210f64b3"
                .parse()
                .unwrap();
            StakeAddress::new(Network::Mainnet, true, hash)
        };

        let stake_2 = {
            let hash = "fe0e6d6312ffb2055509b8815ddd36e01f7c696f6e2e88d7fe4bc1f6"
                .parse()
                .unwrap();
            StakeAddress::new(Network::Mainnet, true, hash)
        };

        let stake_3 = {
            let hash = "276fd18711931e2c0e21430192dbeac0e458093cd9d1fcd7210f64b3"
                .parse()
                .unwrap();
            StakeAddress::new(Network::Preview, false, hash)
        };

        let addresses = [
            (stake_1.clone(), vec![
                StakeAddressRange {
                    active_from: 10.into(),
                    inactive_from: Some(20.into()),
                },
                StakeAddressRange {
                    active_from: 50.into(),
                    inactive_from: None,
                },
            ]),
            (stake_2.clone(), vec![StakeAddressRange {
                active_from: 50.into(),
                inactive_from: Some(70.into()),
            }]),
            (stake_3.clone(), vec![StakeAddressRange {
                active_from: 10.into(),
                inactive_from: None,
            }]),
        ]
        .iter()
        .cloned()
        .collect();
        let history = StakeAddressesHistory { addresses };

        let sorted = history.sorted();
        let expected = [
            (stake_1.clone(), StakeAddressRange {
                active_from: 10.into(),
                inactive_from: Some(20.into()),
            }),
            (stake_3, StakeAddressRange {
                active_from: 10.into(),
                inactive_from: None,
            }),
            (stake_2, StakeAddressRange {
                active_from: 50.into(),
                inactive_from: Some(70.into()),
            }),
            (stake_1, StakeAddressRange {
                active_from: 50.into(),
                inactive_from: None,
            }),
        ];
        assert_eq!(sorted.as_slice(), expected);
    }
}
