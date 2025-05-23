//! Configuration for the Mithril Snapshot used by the follower.

use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::bail;
use cardano_blockchain_types::{Network, Point};
use dashmap::DashMap;
use futures::future::join_all;
use strum::IntoEnumIterator;
use tokio::{
    fs::{self},
    io::{self},
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{debug, error};

use crate::{
    error::{Error, Result},
    mithril_snapshot_data::{latest_mithril_snapshot_id, SnapshotData},
    mithril_snapshot_sync::background_mithril_update,
    snapshot_id::SnapshotId,
    stats,
    turbo_downloader::DlConfig,
};

/// Type we use to manage the Sync Task handle map.
type SyncMap = DashMap<Network, Mutex<Option<JoinHandle<()>>>>;

/// Handle to the mithril sync thread. One for each Network ONLY.
static SYNC_JOIN_HANDLE_MAP: LazyLock<SyncMap> = LazyLock::new(|| {
    let map = DashMap::new();
    for network in Network::iter() {
        map.insert(network, Mutex::new(None));
    }
    map
});

/// Subdirectory where we unpack archives temporarily.
const TMP_SUB_DIR: &str = "tmp";

/// Message we send when Mithril Snapshot updates
#[derive(Debug)]
pub(crate) struct MithrilUpdateMessage {
    /// The largest block on the mithril snapshot.
    pub tip: Point,
    /// The block immediately before it.
    pub previous: Point,
}

/// Configuration used for the Mithril Snapshot downloader.
#[derive(Clone, Debug)]
pub struct MithrilSnapshotConfig {
    /// What Blockchain network are we configured for.
    pub chain: Network,
    /// Path to the Mithril snapshot the follower should use.
    /// Note: this is a base directory.  The Actual data will be stored under here.
    /// archive downloads -> `<mithril_snapshot_path>/dl`
    /// unpacked snapshots -> `<mithril_snapshot_path>/<immutable-file-no>`
    /// extracting snapshots -> `<mithril_snapshot_path>/tmp`
    pub path: PathBuf,
    /// Address of the Mithril Aggregator to use to find the latest snapshot data to
    /// download.
    pub aggregator_url: String,
    /// The Genesis Key needed for a network to do Mithril snapshot validation.
    pub genesis_key: String,
    /// Downloader configuration.
    pub dl_config: Option<DlConfig>,
}

impl MithrilSnapshotConfig {
    /// Sets the defaults for a given cardano network.
    /// Each network has a different set of defaults, so no single "default" can apply.
    /// This function is preferred to the `default()` standard function.
    #[must_use]
    pub fn default_for(chain: Network) -> Self {
        Self {
            chain,
            path: chain.default_mithril_path(),
            aggregator_url: chain.default_mithril_aggregator(),
            genesis_key: chain.default_mithril_genesis_key(),
            dl_config: None,
        }
    }

    /// Set a custom downloader configuration.
    pub fn with_dl_config(mut self, config: DlConfig) -> Self {
        self.dl_config = Some(config);
        self
    }

    /// Try and recover the latest snapshot id from the files on disk.
    #[must_use]
    pub(crate) async fn recover_latest_snapshot_id(&self) -> Option<SnapshotId> {
        // Can we read directory entries from the base path, if not then there is no latest
        // snapshot.
        let path = self.path.clone();
        debug!("Recovering latest snapshot id from {:?}", &path);

        let Ok(mut entries) = fs::read_dir(&self.path).await else {
            error!(
                "Getting latest snapshot failed: Can't read entries from {}",
                self.path.to_string_lossy()
            );
            return None;
        };

        let mut latest_immutable_file: u64 = 0; // Can't have a 0 file.
        let mut latest_path = PathBuf::new();

        loop {
            // Get the next entry, stop on any error, or no entries left.
            let Ok(Some(entry)) = entries.next_entry().await else {
                break;
            };

            if let Some(immutable_file) = SnapshotId::parse_path(&entry.path()) {
                if immutable_file > latest_immutable_file {
                    latest_immutable_file = immutable_file;
                    latest_path = entry.path();
                }
            }
        }

        if latest_immutable_file > 0 {
            return SnapshotId::try_new(self.chain, &latest_path).await;
        }

        None
    }

    /// Activate the tmp mithril path to a numbered snapshot path.
    /// And then remove any left over files in download or the tmp path, or old snapshots.
    pub(crate) async fn activate(&self, snapshot_number: u64) -> io::Result<PathBuf> {
        let new_path = self.mithril_path(snapshot_number);
        let latest_id = latest_mithril_snapshot_id(self.chain);

        debug!(
            "Activating snapshot: {} {} {:?}",
            snapshot_number,
            new_path.to_string_lossy(),
            latest_id
        );

        // Can't activate anything if the tmp directory does not exist.
        if !self.tmp_path().is_dir() {
            error!("No tmp path found to activate.");
            return Err(io::Error::new(io::ErrorKind::NotFound, "No tmp path found"));
        }

        // Check if we would actually be making a newer snapshot active. (Should never fail, but
        // check anyway.)
        if latest_id >= snapshot_number {
            error!("Latest snapshot {latest_id:?} is >= than requested snapshot {snapshot_number}");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Latest snapshot is newer or equal",
            ));
        }

        // Rename the tmp path to the new numbered path.
        fs::rename(self.tmp_path(), &new_path).await?;

        Ok(new_path)
    }

    /// Cleanup the tmp mithril path, all old mithril paths and the dl path.
    /// Removes those directories if they exist and all the files they contain.
    pub(crate) async fn cleanup(&self) -> io::Result<()> {
        let mut cleanup_tasks = Vec::new();

        // Cleanup up the tmp path. (Shouldn't normally exist, but clean it anyway)
        let tmp = self.tmp_path();
        if tmp.exists() {
            debug!("Cleaning up TMP @ {}", tmp.display());
            cleanup_tasks.push(fs::remove_dir_all(tmp.clone()));
        }

        // Cleanup all numbered paths which are not this latest path
        match fs::read_dir(&self.path).await {
            Err(err) => {
                error!(
                    "Unexpected failure reading entries in the mithril path {} : {}",
                    self.path.to_string_lossy(),
                    err
                );
            },
            Ok(mut entries) => {
                // Get latest mithril snapshot path and number.
                let latest_snapshot = latest_mithril_snapshot_id(self.chain);

                loop {
                    // Get the next entry, stop on any error, or no entries left.
                    let Ok(Some(entry)) = entries.next_entry().await else {
                        break;
                    };

                    // If None, its not a snapshot path, so continue.
                    if let Some(this_snapshot) = SnapshotId::new(&entry.path(), Point::ORIGIN) {
                        // Don't do anything with the latest snapshot.
                        // Comparison does NOT use `tip` so we construct a temporary ID without it.
                        if this_snapshot != latest_snapshot {
                            debug!(
                                "Cleaning up non-latest snapshot @ {}",
                                entry.path().display()
                            );
                            cleanup_tasks.push(fs::remove_dir_all(entry.path()));
                        }
                    };
                }
            },
        }

        for result in join_all(cleanup_tasks).await {
            match result {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to cleanup snapshot:  {err:?}");
                },
            }
        }

        Ok(())
    }

    /// Deduplicate a file in the tmp directory vs its equivalent in the current snapshot.
    ///
    /// This does not check if they SHOULD be de-duped, only de-dupes the files specified.
    pub(crate) fn dedup_tmp(
        &self, tmp_file: &Path, latest_snapshot: &SnapshotData,
    ) -> anyhow::Result<()> {
        // Get the matching src file in the latest mithril snapshot to compare against.
        let snapshot_path = latest_snapshot.id().as_ref();
        let tmp_path = self.tmp_path();

        let Ok(relative_file) = tmp_file.strip_prefix(&tmp_path) else {
            error!("Failed to get relative path of file.");
            bail!("Failed to strip prefix: {tmp_path:?}");
        };

        // IF we make it here, the files are identical, so we can de-dup them safely.
        // Remove the tmp file momentarily.
        if tmp_file.exists() {
            if let Err(error) = std::fs::remove_file(tmp_file) {
                error!(
                    "Error removing tmp file  {} :  {}",
                    tmp_file.to_string_lossy(),
                    error
                );
                bail!("Failed to remove tmp file: {tmp_file:?}");
            }
        }

        let src_file = snapshot_path.join(relative_file);
        let src_file = src_file.as_path();
        // Hardlink the src file to the tmp file.
        if let Some(parent) = tmp_file.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                error!("Error creating parent dir {parent:?} for tmp file {tmp_file:?}: {error}");
            }
        }
        if let Err(error) = std::fs::hard_link(src_file, tmp_file) {
            error!(
                "Error linking src file {} to tmp file {} : {}",
                src_file.to_string_lossy(),
                tmp_file.to_string_lossy(),
                error
            );
            bail!("Failed to link src file: {src_file:?}");
        }

        // And if we made it here, file was successfully de-duped.  YAY.
        debug!("DeDup OK: {tmp_file:?}");
        Ok(())
    }

    /// Returns the path to Latest Tmp Snapshot Data.
    /// Will use a path relative to mithril data path.
    #[must_use]
    pub(crate) fn tmp_path(&self) -> PathBuf {
        let mut snapshot_path = self.path.clone();
        snapshot_path.push(TMP_SUB_DIR);
        snapshot_path
    }

    /// Returns the path to the Numbered Snapshot Data.
    /// Will use a path relative to mithril data path.
    #[must_use]
    pub(crate) fn mithril_path(&self, snapshot_number: u64) -> PathBuf {
        let mut snapshot_path = self.path.clone();
        snapshot_path.push(snapshot_number.to_string());
        snapshot_path
    }

    /// Check if the Mithril Snapshot Path is valid and usable.
    async fn validate_path(&self) -> Result<()> {
        let path = self.path.clone();
        debug!(
            path = path.to_string_lossy().to_string(),
            "Validating Mithril Snapshot Path"
        );

        // If the path does not exist, try and make it.
        if !path.exists() {
            // Try and make the directory.
            fs::create_dir_all(&path)
                .await
                .map_err(|e| Error::MithrilSnapshotDirectoryCreation(path.clone(), e))?;
        }

        // If the path is NOT a directory, then we can't use it.
        if !path.is_dir() {
            return Err(Error::MithrilSnapshotDirectoryNotFound(
                path.display().to_string(),
            ));
        }

        // If the directory is not writable then we can't use
        if !check_writable(&path) {
            return Err(Error::MithrilSnapshotDirectoryNotWritable(path.clone()));
        }

        Ok(())
    }

    /// Validate the Genesis VKEY is at least the correct kind of data.
    fn validate_genesis_vkey(&self) -> Result<()> {
        // First sanitize the vkey by removing all whitespace and make sure its actually valid
        // hex.
        let vkey = remove_whitespace(&self.genesis_key);
        if !is_hex(&vkey) {
            return Err(Error::MithrilGenesisVKeyNotHex(self.chain));
        }

        Ok(())
    }

    /// Validate the Aggregator is resolvable and responsive.
    async fn validate_aggregator_url(&self) -> Result<()> {
        let url = self.aggregator_url.clone();
        let key = self.genesis_key.clone();

        debug!(url = url, "Validating Aggregator URL");

        // Not configured already, and not already in use, so make sure its valid.
        // We do this by trying to use it to get a list of snapshots.
        let client = mithril_client::ClientBuilder::aggregator(&url, &key)
            .build()
            .map_err(|e| Error::MithrilClient(self.chain, url.clone(), e))?;

        let snapshots = client
            .cardano_database()
            .list()
            .await
            .map_err(|e| Error::MithrilClient(self.chain, url.clone(), e))?;

        // Check we have a snapshot, and its for our network.
        match snapshots.first() {
            Some(snapshot_info) => {
                let network = snapshot_info.network.as_str();
                let _aggregator_network = Network::from_str(network).map_err(|_err| {
                    Error::MithrilClientNetworkMismatch(self.chain, network.to_string())
                })?;
            },
            None => return Err(Error::MithrilClientNoSnapshots(self.chain, url)),
        }

        Ok(())
    }

    /// Validate the mithril sync configuration is correct.
    pub(crate) async fn validate(&self) -> Result<()> {
        // Validate the path exists and is a directory, and is writable.
        self.validate_path().await?;
        // Validate the genesis vkey is valid.
        self.validate_genesis_vkey()?;
        // Validate the Aggregator is valid and responsive.
        self.validate_aggregator_url().await?;

        Ok(())
    }

    /// Run a Mithril Follower for the given network and configuration.
    pub(crate) async fn run(&self) -> Result<mpsc::Receiver<MithrilUpdateMessage>> {
        debug!(
            chain = self.chain.to_string(),
            "Mithril Auto-update : Starting"
        );

        // Start the Mithril Sync - IFF its not already running.
        let lock_entry = match SYNC_JOIN_HANDLE_MAP.get(&self.chain) {
            None => {
                error!("Join Map improperly initialized: Missing {}!!", self.chain);
                return Err(Error::Internal); // Should not get here.
            },
            Some(entry) => entry,
        };
        let mut locked_handle = lock_entry.value().lock().await;

        if (*locked_handle).is_some() {
            debug!("Mithril Already Running for {}", self.chain);
            return Err(Error::MithrilSnapshotSyncAlreadyRunning(self.chain));
        }

        self.validate().await?;

        // Create a Queue we use to signal the Live Blockchain Follower that the Mithril Snapshot
        // TIP has changed.
        // Given how long even the smallest blockchains take to download, a queue depth of 2 is
        // plenty.
        let (tx, rx) = mpsc::channel::<MithrilUpdateMessage>(2);

        // let handle = tokio::spawn(background_mithril_update(chain, self.clone(), tx));
        let config = self.clone();
        *locked_handle = Some(tokio::spawn(async move {
            stats::start_thread(
                config.chain,
                stats::thread::name::MITHRIL_SNAPSHOT_UPDATER,
                true,
            );
            background_mithril_update(config.clone(), tx).await;
            stats::stop_thread(config.chain, stats::thread::name::MITHRIL_SNAPSHOT_UPDATER);
        }));

        // sync_map.insert(chain, handle);
        debug!(
            chain = self.chain.to_string(),
            "Mithril Auto-update : Started"
        );

        Ok(rx)
    }
}

/// Check that a given mithril snapshot path and everything in it is writable.
/// We don't care why its NOT writable, just that it is either all writable, or not.
/// Will return false on the first detection of a read only file or directory.
fn check_writable(path: &Path) -> bool {
    // Check the permissions of the current path
    if let Ok(metadata) = path.metadata() {
        if metadata.permissions().readonly() {
            return false;
        }
    }

    // Can't read the directory for any reason, so can't write to the directory.
    let path_iterator = match path.read_dir() {
        Err(_) => return false,
        Ok(entries) => entries,
    };

    // Recursively check the contents of the directory
    for entry in path_iterator {
        let Ok(entry) = entry else { return false };

        // If the entry is a directory, recursively check its permissions
        // otherwise just check we could re-write it.
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                // This can NOT be combined with the `if` above.
                // Doing so will cause the `else` to run on non-writable directories.
                // Which is wrong.
                if !check_writable(&entry.path()) {
                    return false;
                }
            } else {
                // If its not a directory then it must be a file.
                if metadata.permissions().readonly() {
                    return false;
                }
            }
        } else {
            // Can't identify the file type, so we can't dedup it.
            return false;
        }
    }
    // Otherwise we could write everything we scanned.
    true
}

/// Remove whitespace from a string and return the new string
fn remove_whitespace(s: &str) -> String {
    s.chars()
        .filter(|&c| !c.is_ascii_whitespace())
        .collect::<String>()
}

/// Check if a string is an even number of hex digits.
fn is_hex(s: &str) -> bool {
    s.chars().count() % 2 == 0 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_for() {
        let network = Network::Preprod;
        let config = MithrilSnapshotConfig::default_for(network);

        assert_eq!(config.chain, network);
        assert_eq!(config.path, network.default_mithril_path());
        assert_eq!(config.aggregator_url, network.default_mithril_aggregator());
        assert_eq!(config.genesis_key, network.default_mithril_genesis_key());
    }

    #[tokio::test]
    async fn test_validate_genesis_vkey() {
        let config = MithrilSnapshotConfig {
            chain: Network::Preprod,
            path: PathBuf::new(),
            aggregator_url: String::new(),
            genesis_key: "1234abcd".to_string(),
            dl_config: None,
        };

        assert!(config.validate_genesis_vkey().is_ok());

        let invalid_config = MithrilSnapshotConfig {
            chain: Network::Preprod,
            path: PathBuf::new(),
            aggregator_url: String::new(),
            genesis_key: "1234abcz".to_string(),
            dl_config: None,
        };

        assert!(invalid_config.validate_genesis_vkey().is_err());
    }
}
