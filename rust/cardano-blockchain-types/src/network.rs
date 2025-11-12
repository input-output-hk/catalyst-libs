//! Enum of possible Cardano networks.

use std::{ffi::OsStr, path::PathBuf};

use catalyst_types::conversion::from_saturating;
use chrono::{DateTime, Utc};
use pallas_addresses::Network as PallasNetwork;
use pallas_primitives::types::network_constant::{
    MAINNET_MAGIC, PRE_PRODUCTION_MAGIC, PREVIEW_MAGIC,
};
use pallas_traverse::wellknown::GenesisValues;
use tracing::debug;

use crate::Slot;

/// Default name of the executable if we can't derive it.
pub(crate) const DEFAULT_EXE_NAME: &str = "cardano_chain_follower";
/// ENV VAR name for the data path.
pub(crate) const ENVVAR_MITHRIL_DATA_PATH: &str = "MITHRIL_DATA_PATH";
/// ENV VAR name for the executable name.
pub(crate) const ENVVAR_MITHRIL_EXE_NAME: &str = "MITHRIL_EXE_NAME";

/// Enum of possible Cardano networks.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, strum::VariantNames, strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum Network {
    /// Cardano mainnet network.
    Mainnet,
    /// Cardano pre-production network.
    Preprod,
    /// Cardano preview network.
    Preview,
    /// Cardano devnet network.
    Devnet {
        /// Mithril signature genesis key for a blockchain.
        genesis_key: String,
        /// Cardano blockchain networking magic number genesis block setting
        magic: u64,
        /// Cardano blockchain network id genesis block setting
        network_id: u64,
        /// Cardano byron epoch length genesis block setting
        byron_epoch_length: u32,
        /// Cardano byron slot length genesis block setting
        byron_slot_length: u32,
        /// Cardano byron known slot genesis block setting
        byron_known_slot: u64,
        /// Cardano byron known hash genesis block setting
        byron_known_hash: String,
        /// Cardano byron known time genesis block setting
        byron_known_time: u64,
        /// Cardano shelley epoch length genesis block setting
        shelley_epoch_length: u32,
        /// Cardano shelley slot length genesis block setting
        shelley_slot_length: u32,
        /// Cardano shelley known slot genesis block setting
        shelley_known_slot: u64,
        /// Cardano shelley known hash genesis block setting
        shelley_known_hash: String,
        /// Cardano shelley known time genesis block setting
        shelley_known_time: u64,
    },
}

// Mainnet Defaults.
/// Mainnet Default Public Cardano Relay.
const DEFAULT_MAINNET_RELAY: &str = "backbone.cardano.iog.io:3001";
/// Main-net Mithril Signature genesis vkey.
const DEFAULT_MAINNET_MITHRIL_GENESIS_KEY: &str = include_str!("data/mainnet-genesis.vkey");
/// Default Mithril Aggregator to use.
const DEFAULT_MAINNET_MITHRIL_AGGREGATOR: &str =
    "https://aggregator.release-mainnet.api.mithril.network/aggregator";

// Preprod Defaults
/// Preprod Default Public Cardano Relay.
const DEFAULT_PREPROD_RELAY: &str = "preprod-node.play.dev.cardano.org:3001";
/// Preprod network Mithril Signature genesis vkey.
const DEFAULT_PREPROD_MITHRIL_GENESIS_KEY: &str = include_str!("data/preprod-genesis.vkey");
/// Default Mithril Aggregator to use.
const DEFAULT_PREPROD_MITHRIL_AGGREGATOR: &str =
    "https://aggregator.release-preprod.api.mithril.network/aggregator";

// Preview Defaults
/// Preview Default Public Cardano Relay.
const DEFAULT_PREVIEW_RELAY: &str = "preview-node.play.dev.cardano.org:3001";
/// Preview network Mithril Signature genesis vkey.
const DEFAULT_PREVIEW_MITHRIL_GENESIS_KEY: &str = include_str!("data/preview-genesis.vkey");
/// Default Mithril Aggregator to use.
const DEFAULT_PREVIEW_MITHRIL_AGGREGATOR: &str =
    "https://aggregator.pre-release-preview.api.mithril.network/aggregator";

// Devnet Defaults
/// Devnet Default Cardano Relay.
const DEFAULT_DEVNET_RELAY: &str = "127.0.0.1:3001";
/// Default Mithril Aggregator to use.
const DEFAULT_DEVNET_MITHRIL_AGGREGATOR: &str = "http://127.0.0.1:8080/aggregator";

impl Network {
    /// Get the default Relay for a blockchain network.
    #[must_use]
    pub fn default_relay(&self) -> String {
        match self {
            Network::Mainnet => DEFAULT_MAINNET_RELAY.to_string(),
            Network::Preprod => DEFAULT_PREPROD_RELAY.to_string(),
            Network::Preview => DEFAULT_PREVIEW_RELAY.to_string(),
            Network::Devnet { .. } => DEFAULT_DEVNET_RELAY.to_string(),
        }
    }

    /// Get the default aggregator for a blockchain.
    #[must_use]
    pub fn default_mithril_aggregator(&self) -> String {
        match self {
            Network::Mainnet => DEFAULT_MAINNET_MITHRIL_AGGREGATOR.to_string(),
            Network::Preprod => DEFAULT_PREPROD_MITHRIL_AGGREGATOR.to_string(),
            Network::Preview => DEFAULT_PREVIEW_MITHRIL_AGGREGATOR.to_string(),
            Network::Devnet { .. } => DEFAULT_DEVNET_MITHRIL_AGGREGATOR.to_string(),
        }
    }

    /// Get the default Mithril Signature genesis key for a blockchain.
    #[must_use]
    pub fn default_mithril_genesis_key(&self) -> String {
        match self {
            Network::Mainnet => DEFAULT_MAINNET_MITHRIL_GENESIS_KEY.to_string(),
            Network::Preprod => DEFAULT_PREPROD_MITHRIL_GENESIS_KEY.to_string(),
            Network::Preview => DEFAULT_PREVIEW_MITHRIL_GENESIS_KEY.to_string(),
            Network::Devnet { genesis_key, .. } => (*genesis_key).to_string(),
        }
    }

    /// Get the default storage location for mithril snapshots.
    /// Defaults to: `<platform data_local_dir>/<exe name>/mithril/<network>`
    pub fn default_mithril_path(&self) -> PathBuf {
        // Get the base path for storing Data.
        // IF the ENV var is set, use that.
        // Otherwise use the system default data path for an application.
        // All else fails default to "/var/lib"
        let mut base_path = std::env::var(ENVVAR_MITHRIL_DATA_PATH).map_or_else(
            |_| dirs::data_local_dir().unwrap_or("/var/lib".into()),
            PathBuf::from,
        );

        // Get the Executable name for the data path.
        // IF the ENV var is set, use it, otherwise try and get it from the exe itself.
        // Fallback to using a default exe name if all else fails.
        let exe_name = std::env::var(ENVVAR_MITHRIL_EXE_NAME).unwrap_or(
            std::env::current_exe()
                .unwrap_or(DEFAULT_EXE_NAME.into())
                .file_name()
                .unwrap_or(OsStr::new(DEFAULT_EXE_NAME))
                .to_string_lossy()
                .to_string(),
        );

        // <base path>/<exe name>
        base_path.push(exe_name);

        // Put everything in a `mithril` sub directory.
        base_path.push("mithril");

        // <base path>/<exe name>/<network>
        base_path.push(self.to_string());

        debug!(
            chain = self.to_string(),
            path = base_path.to_string_lossy().to_string(),
            "DEFAULT Mithril Data Path",
        );

        // Return the final path
        base_path
    }

    /// Return genesis values for given network
    #[must_use]
    pub fn genesis_values(&self) -> GenesisValues {
        match self {
            Network::Mainnet => GenesisValues::mainnet(),
            Network::Preprod => GenesisValues::preprod(),
            Network::Preview => GenesisValues::preview(),
            Network::Devnet {
                magic,
                network_id,
                byron_epoch_length,
                byron_slot_length,
                byron_known_slot,
                byron_known_hash,
                byron_known_time,
                shelley_epoch_length,
                shelley_slot_length,
                shelley_known_slot,
                shelley_known_hash,
                shelley_known_time,
                ..
            } => {
                GenesisValues {
                    magic: *magic,
                    network_id: *network_id,
                    byron_epoch_length: *byron_epoch_length,
                    byron_slot_length: *byron_slot_length,
                    byron_known_slot: *byron_known_slot,
                    byron_known_hash: (*byron_known_hash).to_string(),
                    byron_known_time: *byron_known_time,
                    shelley_epoch_length: *shelley_epoch_length,
                    shelley_slot_length: *shelley_slot_length,
                    shelley_known_slot: *shelley_known_slot,
                    shelley_known_hash: (*shelley_known_hash).to_string(),
                    shelley_known_time: *shelley_known_time,
                }
            },
        }
    }

    /// Return networking magic number.
    #[must_use]
    pub fn magic_number(&self) -> u64 {
        match self {
            Network::Mainnet => MAINNET_MAGIC,
            Network::Preprod => PRE_PRODUCTION_MAGIC,
            Network::Preview => PREVIEW_MAGIC,
            Network::Devnet { magic, .. } => *magic,
        }
    }

    /// Convert a given slot# to its Wall Time for a Blockchain network.
    #[must_use]
    pub fn slot_to_time(
        &self,
        slot: Slot,
    ) -> DateTime<Utc> {
        let genesis = self.genesis_values();
        let wall_clock = genesis.slot_to_wallclock(slot.into());

        let raw_time: i64 = wall_clock.try_into().unwrap_or(i64::MAX);
        DateTime::from_timestamp(raw_time, 0).unwrap_or(DateTime::<Utc>::MAX_UTC)
    }

    /// Convert an arbitrary time to a slot.
    ///
    /// If the given time predates the blockchain, will return None.
    ///
    /// The Slot does not have to be a valid slot present in the blockchain.
    #[must_use]
    pub fn time_to_slot(
        &self,
        time: DateTime<Utc>,
    ) -> Option<Slot> {
        let genesis = self.genesis_values();

        let byron_start_time =
            DateTime::<Utc>::from_timestamp(from_saturating(genesis.byron_known_time), 0)?;
        let shelley_start_time =
            DateTime::<Utc>::from_timestamp(from_saturating(genesis.shelley_known_time), 0)?;

        // determine if the given time is in Byron or Shelley era.
        if time < byron_start_time {
            return None;
        }

        let (elapsed_slots, era_known_slot) = if time < shelley_start_time {
            // Byron era
            let time_diff = time.signed_duration_since(byron_start_time);
            let elapsed_slots = time_diff
                .num_seconds()
                .checked_div(i64::from(genesis.byron_slot_length))?;
            (elapsed_slots, genesis.byron_known_slot)
        } else {
            // Shelley era
            let time_diff = time.signed_duration_since(shelley_start_time);
            let elapsed_slots = time_diff
                .num_seconds()
                .checked_div(i64::from(genesis.shelley_slot_length))?;
            (elapsed_slots, genesis.shelley_known_slot)
        };

        let era_known_slot = i64::try_from(era_known_slot).ok()?;
        let slot = elapsed_slots.checked_add(era_known_slot)?;
        Some(Slot::from_saturating(slot))
    }
}

impl From<Network> for PallasNetwork {
    fn from(value: Network) -> Self {
        PallasNetwork::from(&value)
    }
}

impl From<&Network> for PallasNetwork {
    fn from(value: &Network) -> Self {
        match value {
            Network::Mainnet => PallasNetwork::Mainnet,
            Network::Preprod | Network::Preview | Network::Devnet { .. } => PallasNetwork::Testnet,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use strum::VariantNames;

    use super::*;

    #[test]
    fn test_variant_to_string() {
        assert_eq!(Network::Mainnet.to_string(), "mainnet");
        assert_eq!(Network::Preprod.to_string(), "preprod");
        assert_eq!(Network::Preview.to_string(), "preview");
        assert_eq!(
            Network::Devnet {
                genesis_key: "genesis_key".into(),
                magic: 0,
                network_id: 0,
                byron_epoch_length: 0,
                byron_slot_length: 0,
                byron_known_slot: 0,
                byron_known_hash: "byron_known_hash".into(),
                byron_known_time: 0,
                shelley_epoch_length: 0,
                shelley_slot_length: 0,
                shelley_known_slot: 0,
                shelley_known_hash: "shelley_known_hash".into(),
                shelley_known_time: 0
            }
            .to_string(),
            "devnet"
        );

        let expected_names = ["mainnet", "preprod", "preview", "devnet"];
        assert_eq!(Network::VARIANTS, expected_names);
    }

    #[test]
    fn test_time_to_slot_before_blockchain() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        let before_blockchain = Utc
            .timestamp_opt(i64::try_from(genesis.byron_known_time).unwrap() - 1, 0)
            .unwrap();

        assert_eq!(network.time_to_slot(before_blockchain), None);
    }

    #[test]
    fn test_time_to_slot_byron_era() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        let byron_start_time = Utc
            .timestamp_opt(i64::try_from(genesis.byron_known_time).unwrap(), 0)
            .unwrap();
        let byron_slot_length = i64::from(genesis.byron_slot_length);

        // a time in the middle of the Byron era.
        let time = byron_start_time + chrono::Duration::seconds(byron_slot_length * 100);
        let expected_slot = Slot::from_saturating(genesis.byron_known_slot + 100);

        assert_eq!(network.time_to_slot(time), Some(expected_slot));
    }

    #[test]
    fn test_time_to_slot_transition_to_shelley() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        let shelley_start_time = Utc
            .timestamp_opt(i64::try_from(genesis.shelley_known_time).unwrap(), 0)
            .unwrap();
        let byron_slot_length = i64::from(genesis.byron_slot_length);

        // a time just before Shelley era starts.
        let time = shelley_start_time - chrono::Duration::seconds(1);
        let elapsed_slots = (time
            - Utc
                .timestamp_opt(i64::try_from(genesis.byron_known_time).unwrap(), 0)
                .unwrap())
        .num_seconds()
            / byron_slot_length;
        let expected_slot = Slot::from_saturating(
            genesis.byron_known_slot + from_saturating::<u64, i64>(elapsed_slots),
        );

        assert_eq!(network.time_to_slot(time), Some(expected_slot));
    }

    #[test]
    fn test_time_to_slot_shelley_era() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        let shelley_start_time = Utc
            .timestamp_opt(i64::try_from(genesis.shelley_known_time).unwrap(), 0)
            .unwrap();
        let shelley_slot_length = i64::from(genesis.shelley_slot_length);

        // a time in the middle of the Shelley era.
        let time = shelley_start_time + chrono::Duration::seconds(shelley_slot_length * 200);
        let expected_slot = Slot::from_saturating(genesis.shelley_known_slot + 200);

        assert_eq!(network.time_to_slot(time), Some(expected_slot));
    }

    #[test]
    fn test_slot_to_time_to_slot_consistency() {
        let network = Network::Mainnet;

        // a few arbitrary slots in different ranges.
        let slots_to_test = vec![0, 10_000, 1_000_000, 50_000_000];

        for slot in slots_to_test {
            let time = network.slot_to_time(slot.into());
            let calculated_slot = network.time_to_slot(time);

            assert_eq!(
                calculated_slot,
                Some(Slot::from_saturating(slot)),
                "Failed for slot: {slot}"
            );
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_time_to_slot_to_time_consistency() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        // Byron, Shelley, and Conway.
        let times_to_test = vec![
            Utc.timestamp_opt(i64::try_from(genesis.byron_known_time).unwrap() + 100, 0)
                .unwrap(),
            Utc.timestamp_opt(
                i64::try_from(genesis.shelley_known_time).unwrap() + 1_000,
                0,
            )
            .unwrap(),
            Utc.timestamp_opt(
                i64::try_from(genesis.shelley_known_time).unwrap() + 10_000_000,
                0,
            )
            .unwrap(),
        ];

        for time in times_to_test {
            if let Some(slot) = network.time_to_slot(time) {
                let calculated_time = network.slot_to_time(slot);

                assert_eq!(
                    calculated_time.timestamp(),
                    time.timestamp(),
                    "Failed for time: {time}"
                );
            } else {
                panic!("time_to_slot returned None for a valid time: {time}");
            }
        }
    }

    #[test]
    fn test_conway_era_time_to_slot_and_back() {
        let network = Network::Mainnet;
        let genesis = network.genesis_values();

        // a very late time, far in the Conway era.
        let conway_time = Utc
            .timestamp_opt(
                i64::try_from(genesis.shelley_known_time).unwrap() + 20_000_000,
                0,
            )
            .unwrap();

        let slot = network.time_to_slot(conway_time);
        assert!(
            slot.is_some(),
            "Failed to calculate slot for Conway era time"
        );

        let calculated_time = network.slot_to_time(slot.unwrap());

        assert_eq!(
            calculated_time.timestamp(),
            conway_time.timestamp(),
            "Inconsistency for Conway era time"
        );
    }
}
