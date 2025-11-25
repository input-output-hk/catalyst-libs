//! Hermes IPFS
//!
//! Provides support for storage, and `PubSub` functionality.
//!
//! # Cross-Platform Support
//!
//! This library supports both native (desktop/server) and WASM targets.
//! The key differences between these platforms are:
//!
//! ## Native Targets (Linux, macOS, Windows)
//! - **Storage**: Uses filesystem-backed storage via `tokio::fs`
//! - **File handling**: Supports reading files from disk using `std::path::PathBuf`
//! - **API**: `AddIpfsFile::Path` variant available for direct file system access
//!
//! ## WASM Targets (wasm32 architecture)
//! - **Storage**: Uses alternative storage backends (e.g., IndexedDB in web environments)
//! - **File handling**: Only supports in-memory byte streams
//! - **API**: `AddIpfsFile::Path` variant is NOT available; use `AddIpfsFile::Stream` instead
//!
//! **Why these differences?** WASM runtimes typically don't provide direct filesystem access
//! for security and portability reasons. File data must be provided as byte streams.
//!
//! The cross-compilation is achieved using `#[cfg(not(target_arch = "wasm32"))]` attributes
//! to conditionally compile code based on the target architecture.

use std::{convert::Infallible, str::FromStr};

use derive_more::{Display, From, Into};
use futures::{StreamExt, pin_mut, stream::BoxStream};
/// IPFS Content Identifier.
pub use ipld_core::cid::Cid;
/// IPLD
pub use ipld_core::ipld::Ipld;
use libp2p::gossipsub::MessageId as PubsubMessageId;
/// `rust_ipfs` re-export.
pub use rust_ipfs;
/// Server, Client, or Auto mode
pub use rust_ipfs::DhtMode;
/// Server, Client, or Auto mode
pub use rust_ipfs::Ipfs;
/// Multiaddr type.
pub use rust_ipfs::Multiaddr;
/// Peer ID type.
pub use rust_ipfs::PeerId;
/// Peer Info type.
pub use rust_ipfs::p2p::PeerInfo;
/// Enum for specifying paths in IPFS.
pub use rust_ipfs::path::IpfsPath;
/// Storage type for IPFS node.
pub use rust_ipfs::repo::StorageTypes;
use rust_ipfs::{
    GossipsubMessage, NetworkBehaviour, Quorum, ToRecordKey, builder::IpfsBuilder,
    dag::ResolveError, dummy, gossipsub::IntoGossipsubTopic, unixfs::AddOpt,
};

#[derive(Debug, Display, From, Into)]
/// `PubSub` Message ID.
pub struct MessageId(pub PubsubMessageId);

/// Builder type for IPFS Node configuration.
pub struct HermesIpfsBuilder<N>(IpfsBuilder<N>)
where N: NetworkBehaviour<ToSwarm = Infallible> + Send + Sync;

impl Default for HermesIpfsBuilder<dummy::Behaviour> {
    fn default() -> Self {
        Self(IpfsBuilder::new())
    }
}

impl<N> HermesIpfsBuilder<N>
where N: NetworkBehaviour<ToSwarm = Infallible> + Send + Sync
{
    #[must_use]
    /// Create a new` IpfsBuilder`.
    pub fn new() -> Self {
        Self(IpfsBuilder::new())
    }

    #[must_use]
    /// Set the default configuration for the IPFS node.
    pub fn with_default(self) -> Self {
        Self(self.0.with_default())
    }

    #[must_use]
    /// Set the default listener for the IPFS node.
    pub fn set_default_listener(self) -> Self {
        Self(self.0.set_default_listener())
    }

    #[must_use]
    /// Set the storage type for the IPFS node.
    ///
    /// ## Parameters
    ///
    /// * `storage_path` - Path for storage location (or namespace on WASM)
    ///
    /// ## Cross-Platform Behavior
    ///
    /// This method creates different storage backends depending on the target platform:
    ///
    /// ### Native Targets (not(target_arch = "wasm32"))
    /// Creates filesystem-backed storage using `Repo::new_fs()`. The storage_path is used
    /// as a directory path where IPFS blocks and metadata will be stored on disk.
    ///
    /// **Why**: Native platforms have filesystem access via `tokio::fs` and can
    /// persist data directly to disk.
    ///
    /// ### WASM Targets (target_arch = "wasm32")
    /// Creates alternative storage using `Repo::new_idb()`. The storage_path is
    /// converted to a string and used as a namespace identifier for the storage system.
    ///
    /// **Why**: WASM runtimes typically don't provide direct filesystem APIs. Alternative
    /// storage mechanisms are used instead. The path string serves as a logical namespace
    /// rather than an actual file path.
    pub fn set_disk_storage<T: Into<std::path::PathBuf>>(
        self,
        storage_path: T,
    ) -> Self {
        // Native: Use filesystem storage
        #[cfg(not(target_arch = "wasm32"))]
        let repo = { rust_ipfs::repo::Repo::new_fs(storage_path.into()) };

        // WASM: Use alternative storage (e.g., IndexedDB) with path as namespace
        #[cfg(target_arch = "wasm32")]
        let repo = {
            let namespace = storage_path.into().to_string_lossy().to_string();
            rust_ipfs::repo::Repo::new_idb(Some(namespace))
        };

        Self(self.0.set_repo(&repo))
    }

    /// Start the IPFS node.
    ///
    /// ## Errors
    /// Returns an error if the IPFS daemon fails to start.
    pub async fn start(self) -> anyhow::Result<Ipfs> {
        self.0.start().await
    }
}

/// Hermes IPFS Node.
pub struct HermesIpfs {
    /// IPFS node
    node: Ipfs,
}

impl HermesIpfs {
    /// Start a new node.
    ///
    /// ## Returns
    ///
    /// * `HermesIpfs`
    ///
    /// ## Errors
    ///
    /// Returns an error if the IPFS daemon fails to start.
    pub async fn start() -> anyhow::Result<Self> {
        let node: Ipfs = HermesIpfsBuilder::<dummy::Behaviour>::new()
            .with_default()
            .set_default_listener()
            // TODO(saibatizoku): Re-Enable default transport config when libp2p Cert bug is fixed
            // TODO(rafal-ch): TLS is disabled by default, we can enable it by calling
            // on of the `IpfsBuilder::enable_secure...()` flavors.
            //.enable_secure_websocket()
            .start()
            .await?;
        Ok(HermesIpfs { node })
    }

    /// Add a file to IPFS.
    ///
    /// ## Parameters
    ///
    /// * `ipfs_file` - Either a file path (native only) or byte stream (all platforms)
    ///
    /// ## Returns
    ///
    /// * A result with `IpfsPath` pointing to the uploaded content
    ///
    /// ## Errors
    ///
    /// Returns an error if the file fails to upload or (on native) if the file cannot be read.
    ///
    /// ## Cross-Platform Behavior
    ///
    /// This method normalizes file input across platforms before uploading to IPFS:
    ///
    /// ### Native Targets
    /// - Accepts `AddIpfsFile::Path` which contains a filesystem path
    /// - Reads the file from disk using `tokio::fs::read` (requires the `fs` feature)
    /// - Converts to `AddIpfsFile::Stream` with file contents and optional filename
    /// - Then uploads the stream to IPFS
    ///
    /// ### WASM Targets
    /// - Only accepts `AddIpfsFile::Stream` (the Path variant doesn't exist in WASM)
    /// - Directly uploads the provided byte stream to IPFS
    /// - Files must be read into memory by the caller using platform-specific APIs
    ///
    /// **Why this conversion?** The underlying `rust-ipfs` library's `add_unixfs` method
    /// works with streams internally. On native, we provide convenience by handling file
    /// reading, but on WASM there's no filesystem to read from.
    pub async fn add_ipfs_file(
        &self,
        ipfs_file: AddIpfsFile,
    ) -> anyhow::Result<IpfsPath> {
        // On native: Convert Path to Stream by reading from filesystem
        // On WASM: This match only handles Stream since Path doesn't exist
        let ipfs_file = match ipfs_file {
            #[cfg(not(target_arch = "wasm32"))]
            AddIpfsFile::Path(file_path) => {
                // Read file from filesystem using tokio::fs (not available in WASM)
                let file_bytes = tokio::fs::read(&file_path).await.map_err(|e| {
                    anyhow::anyhow!("Failed to read file at {:?}: {}", file_path, e)
                })?;

                // Extract filename for metadata
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());

                // Convert to Stream variant for uploading
                AddIpfsFile::Stream((file_name, file_bytes))
            },
            stream @ AddIpfsFile::Stream(_) => stream,
        };

        let ipfs_path = self.node.add_unixfs(ipfs_file).await?;
        Ok(ipfs_path)
    }

    /// Get a file from IPFS
    ///
    /// ## Parameters
    ///
    /// * `ipfs_path` - `GetIpfsFile(IpfsPath)` Path used to get the file from IPFS.
    ///
    /// ## Returns
    ///
    /// * `A result with Vec<u8>`.
    ///
    /// ## Errors
    ///
    /// Returns an error if the file fails to download.
    pub async fn get_ipfs_file(
        &self,
        ipfs_path: GetIpfsFile,
    ) -> anyhow::Result<Vec<u8>> {
        let stream_bytes = self.node.cat_unixfs(ipfs_path).await?;
        Ok(stream_bytes.to_vec())
    }

    /// Pin content to IPFS.
    ///
    /// ## Parameters
    ///
    /// * `cid` - `Cid` Content identifier to be pinned.
    ///
    /// ## Errors
    ///
    /// Returns an error if pinning fails.
    pub async fn insert_pin(
        &self,
        cid: &Cid,
    ) -> anyhow::Result<()> {
        self.node.insert_pin(cid).await
    }

    /// Checks whether a given block is pinned.
    ///
    /// # Crash unsafety
    ///
    /// Cannot currently detect partially written recursive pins. Those can happen if
    /// [`HermesIpfs::insert_pin`] is interrupted by a crash for example.
    ///
    /// Works correctly only under no-crash situations. Workaround for hitting a crash is
    /// to re-pin any existing recursive pins.
    ///
    /// ## Parameters
    ///
    /// * `cid` - `Cid` Content identifier to be pinned.
    ///
    /// ## Returns
    /// `true` if the block is pinned, `false` if not. See Crash unsafety notes for the
    /// false response.
    ///
    /// ## Errors
    ///
    /// Returns an error if checking pin fails.
    pub async fn is_pinned(
        &self,
        cid: &Cid,
    ) -> anyhow::Result<bool> {
        self.node.is_pinned(cid).await
    }

    /// List all pins in the IPFS node.
    ///
    /// ## Parameters
    /// * `cid` - `Option<Cid>` Optional content identifier to list pins. If `None`, lists
    ///   all pins.
    ///
    /// ## Errors
    /// Returns an error if listing pins fails.
    pub async fn list_pins(&self) -> anyhow::Result<Vec<Cid>> {
        // List all kinds of pins by setting `None` as the argument.
        let pins_stream = self.node.list_pins(None).await;
        pin_mut!(pins_stream);
        let mut pins = vec![];
        while let Some(pinned) = pins_stream.next().await {
            pins.push(pinned?.0);
        }
        Ok(pins)
    }

    /// Remove pinned content from IPFS.
    ///
    /// ## Parameters
    ///
    /// * `cid` - `Cid` Content identifier to be un-pinned.
    ///
    /// ## Errors
    ///
    /// Returns an error if removing pin fails.
    pub async fn remove_pin(
        &self,
        cid: &Cid,
    ) -> anyhow::Result<()> {
        self.node.remove_pin(cid).recursive().await
    }

    /// Stop and exit the IPFS node daemon.
    pub async fn stop(self) {
        self.node.exit_daemon().await;
    }

    /// Returns the peer identity information. If no peer id is supplied the local node
    /// identity is used.
    ///
    /// ## Parameters
    ///
    /// * `peer_id` - `Option<PeerId>`
    ///
    /// ## Errors
    ///
    /// Returns error if peer info cannot be retrieved.
    pub async fn identity(
        &self,
        peer_id: Option<PeerId>,
    ) -> anyhow::Result<PeerId> {
        self.node.identity(peer_id).await.map(|p| p.peer_id)
    }

    /// Add peer to address book.
    ///
    /// ## Parameters
    ///
    /// * `peer_id` - `PeerId`
    /// * `addr` - `Multiaddr`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to add peer.
    pub async fn add_peer(
        &self,
        peer_id: PeerId,
        addr: Multiaddr,
    ) -> anyhow::Result<()> {
        self.node.add_peer((peer_id, addr)).await
    }

    /// List of local listening addresses
    ///
    /// ## Returns
    ///
    /// * `Result<Vec<Multiaddr>>`
    ///
    /// ## Errors
    ///
    /// Returns error if listening addresses cannot be retrieved.
    pub async fn listening_addresses(&self) -> anyhow::Result<Vec<Multiaddr>> {
        self.node.listening_addresses().await
    }

    /// Sets DHT mode in the IPFS node.
    ///
    /// ## Parameters
    ///
    /// * `mode` - `DhtMode`
    ///
    /// ## Returns
    ///
    /// * `Result<()>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to set DHT mode
    pub async fn dht_mode(
        &self,
        mode: DhtMode,
    ) -> anyhow::Result<()> {
        self.node.dht_mode(mode).await
    }

    /// Add DAG data to IPFS.
    ///
    /// ## Parameters
    ///
    /// * `ipld` - `Ipld`
    ///
    /// ## Returns
    ///
    /// * `Result<Cid>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to add DAG content.
    pub async fn dag_put(
        &self,
        ipld: Ipld,
    ) -> anyhow::Result<Cid> {
        self.node.put_dag(ipld).await
    }

    /// Get DAG data from IPFS.
    ///
    /// ## Parameters
    ///
    /// * `path` - `impl Into<IpfsPath>`
    ///
    /// ## Returns
    ///
    /// * `Result<Ipld>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to get DAG content.
    pub async fn dag_get<T: Into<IpfsPath>>(
        &self,
        path: T,
    ) -> Result<Ipld, ResolveError> {
        self.node.get_dag(path).await
    }

    /// Add content to DHT.
    ///
    /// ## Parameters
    ///
    /// * `key` - `impl AsRef<[u8]>`
    /// * `value` - `impl Into<Vec<u8>>`
    ///
    /// ## Returns
    ///
    /// * `Result<()>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to add content to DHT
    pub async fn dht_put(
        &self,
        key: impl AsRef<[u8]>,
        value: impl Into<Vec<u8>>,
    ) -> anyhow::Result<()> {
        self.node.dht_put(key, value.into(), Quorum::One).await
    }

    /// Get content from DHT.
    ///
    /// ## Parameters
    ///
    /// * `key` - `impl AsRef<[u8]>`
    ///
    /// ## Returns
    ///
    /// * `Result<Vec<u8>>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to get content from DHT
    pub async fn dht_get(
        &self,
        key: impl AsRef<[u8]> + ToRecordKey,
    ) -> anyhow::Result<Vec<u8>> {
        let record_stream = self.node.dht_get(key).await?;
        pin_mut!(record_stream);
        let record = record_stream
            .next()
            .await
            .ok_or(anyhow::anyhow!("No record found"))?;
        Ok(record.value)
    }

    /// Add address to bootstrap nodes.
    ///
    /// ## Parameters
    ///
    /// * `address` - `Multiaddr`
    ///
    /// ## Returns
    ///
    /// * `Result<Multiaddr>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to add address to bootstrap nodes
    pub async fn add_bootstrap(
        &self,
        address: Multiaddr,
    ) -> anyhow::Result<Multiaddr> {
        self.node.add_bootstrap(address).await
    }

    /// Bootstrap the IPFS node.
    ///
    /// ## Returns
    ///
    /// * `Result<()>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to retrieve bootstrap the node.
    pub async fn bootstrap(&self) -> anyhow::Result<()> {
        self.node.bootstrap().await
    }

    /// Subscribes to a pubsub topic.
    ///
    /// ## Parameters
    ///
    /// * `topic` - `impl Into<String>`
    ///
    /// ## Returns
    ///
    /// * Stream of `GossipsubEvent`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to subscribe to pubsub topic.
    pub async fn pubsub_subscribe(
        &self,
        topic: impl Into<String>,
    ) -> anyhow::Result<BoxStream<'static, connexa::prelude::GossipsubEvent>> {
        let topic = topic.into();
        self.node.pubsub_subscribe(&topic).await?;
        self.node.pubsub_listener(&topic).await
    }

    /// Unsubscribes from a pubsub topic.
    ///
    /// ## Parameters
    ///
    /// * `topic` - `impl Into<String>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to unsubscribe from pubsub topic.
    pub async fn pubsub_unsubscribe(
        &self,
        topic: impl Into<String> + IntoGossipsubTopic,
    ) -> anyhow::Result<()> {
        self.node.pubsub_unsubscribe(topic).await
    }

    /// Publishes a message to a pubsub topic.
    ///
    /// ## Parameters
    ///
    /// * `topic` - `impl Into<String>`
    /// * `message` - `Vec<u8>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to publish to a pubsub topic.
    pub async fn pubsub_publish(
        &self,
        topic: impl IntoGossipsubTopic,
        message: Vec<u8>,
    ) -> anyhow::Result<()> {
        self.node.pubsub_publish(topic, message).await
    }

    /// Ban peer from node.
    ///
    /// ## Parameters
    ///
    /// * `peer` - `PeerId`
    ///
    /// ## Returns
    ///
    /// * `Result<()>`
    ///
    /// ## Errors
    ///
    /// Returns error if unable to ban peer.
    pub async fn ban_peer(
        &self,
        peer: PeerId,
    ) -> anyhow::Result<()> {
        self.node.ban_peer(peer).await
    }
}

impl From<Ipfs> for HermesIpfs {
    fn from(node: Ipfs) -> Self {
        Self { node }
    }
}

/// File that will be added to IPFS
///
/// ## Cross-Platform Design
///
/// This enum has different variants available depending on the compilation target:
///
/// ### Native Targets (Linux, macOS, Windows)
/// ```rust,ignore
/// // Both variants available:
/// AddIpfsFile::Path(PathBuf)                      // Convenient: read from filesystem
/// AddIpfsFile::Stream((Option<String>, Vec<u8>))  // Explicit: provide bytes
/// ```
///
/// ### WASM Targets (wasm32 architecture)
/// ```rust,ignore
/// // Only Stream variant available:
/// AddIpfsFile::Stream((Option<String>, Vec<u8>))
/// ```
///
/// **Why the difference?**
///
/// - **Native**: Has filesystem access via `std::fs` and `tokio::fs`, so we can provide
///   the `Path` variant as a convenience. The library reads the file for you.
///
/// - **WASM**: Most WASM runtimes don't provide direct filesystem access for security
///   and portability reasons. File data must be obtained through:
///   - Platform-specific APIs
///   - Network operations
///   - Alternative storage systems
///   - In-memory generation
///
///   The caller must obtain the bytes using the appropriate APIs for their environment,
///   then pass them as a Stream. This keeps the library's API honest about what's possible.
pub enum AddIpfsFile {
    /// Path in local disk storage to the file.
    ///
    /// **Only available on non-WASM targets** because WASM runtimes typically don't
    /// provide direct filesystem access.
    ///
    /// The file will be read using `tokio::fs::read` when calling `add_ipfs_file()`.
    #[cfg(not(target_arch = "wasm32"))]
    Path(std::path::PathBuf),

    /// Stream of file bytes with an optional filename.
    ///
    /// **Available on all platforms** (native and WASM).
    ///
    /// - First element: Optional filename for metadata (note: current `rust-ipfs`
    ///   implementation may not preserve names in published files)
    /// - Second element: The actual file contents as bytes
    ///
    /// On WASM, this is the **only** way to add files since there's no direct filesystem access.
    Stream((Option<String>, Vec<u8>)),
}

impl From<AddIpfsFile> for AddOpt {
    fn from(value: AddIpfsFile) -> Self {
        match value {
            #[cfg(not(target_arch = "wasm32"))]
            AddIpfsFile::Path(_) => {
                // Path variants should be converted to Stream in add_ipfs_file() before
                // reaching this conversion. If we hit this, there's a bug in the call chain.
                unreachable!(
                    "Path should be converted to Stream before From<AddIpfsFile> for AddOpt"
                )
            },
            AddIpfsFile::Stream((None, bytes)) => bytes.into(),
            AddIpfsFile::Stream((Some(name), bytes)) => (name, bytes).into(),
        }
    }
}

// Conversion implementations for creating AddIpfsFile from various types
//
// Note: The Path-based conversions (String, PathBuf) are only available on native
// targets because AddIpfsFile::Path doesn't exist in WASM.

/// Convert String to AddIpfsFile::Path on native platforms.
///
/// **Only available on non-WASM targets.**
///
/// Example: `let file = AddIpfsFile::from("/path/to/file.txt".to_string());`
#[cfg(not(target_arch = "wasm32"))]
impl From<String> for AddIpfsFile {
    fn from(value: String) -> Self {
        Self::Path(value.into())
    }
}

/// Convert PathBuf to AddIpfsFile::Path on native platforms.
///
/// **Only available on non-WASM targets.**
///
/// Example: `let file = AddIpfsFile::from(PathBuf::from("/path/to/file.txt"));`
#[cfg(not(target_arch = "wasm32"))]
impl From<std::path::PathBuf> for AddIpfsFile {
    fn from(value: std::path::PathBuf) -> Self {
        Self::Path(value)
    }
}

impl From<Vec<u8>> for AddIpfsFile {
    fn from(value: Vec<u8>) -> Self {
        Self::Stream((None, value))
    }
}

impl From<(String, Vec<u8>)> for AddIpfsFile {
    fn from((name, stream): (String, Vec<u8>)) -> Self {
        Self::Stream((Some(name), stream))
    }
}

impl From<(Option<String>, Vec<u8>)> for AddIpfsFile {
    fn from(value: (Option<String>, Vec<u8>)) -> Self {
        Self::Stream(value)
    }
}

/// Path to get the file from IPFS
pub struct GetIpfsFile(IpfsPath);

impl From<Cid> for GetIpfsFile {
    fn from(value: Cid) -> Self {
        GetIpfsFile(value.into())
    }
}

impl From<IpfsPath> for GetIpfsFile {
    fn from(value: IpfsPath) -> Self {
        GetIpfsFile(value)
    }
}

impl From<GetIpfsFile> for IpfsPath {
    fn from(value: GetIpfsFile) -> Self {
        value.0
    }
}

impl FromStr for GetIpfsFile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GetIpfsFile(s.parse()?))
    }
}

/// `GossipsubEvents` related to subscription state
#[derive(Display, Debug)]
pub enum SubscriptionStatusEvent {
    /// Peer has been subscribed
    Subscribed {
        /// Peer id
        peer_id: PeerId,
    },
    /// Peer has been unsubscribed
    Unsubscribed {
        /// Peer id
        peer_id: PeerId,
    },
}

/// Handle stream of messages from the IPFS pubsub topic
pub fn subscription_stream_task<MH, SH>(
    stream: BoxStream<'static, connexa::prelude::GossipsubEvent>,
    message_handler: MH,
    subscription_handler: SH,
) -> tokio::task::JoinHandle<()>
where
    MH: Fn(GossipsubMessage) + Send + 'static,
    SH: Fn(SubscriptionStatusEvent) + Send + 'static,
{
    tokio::spawn(async move {
        pin_mut!(stream);
        while let Some(msg) = stream.next().await {
            match msg {
                connexa::prelude::GossipsubEvent::Subscribed { peer_id } => {
                    subscription_handler(SubscriptionStatusEvent::Subscribed { peer_id });
                },
                connexa::prelude::GossipsubEvent::Unsubscribed { peer_id } => {
                    subscription_handler(SubscriptionStatusEvent::Unsubscribed { peer_id });
                },
                connexa::prelude::GossipsubEvent::Message { message } => message_handler(message),
            }
        }
    })
}
