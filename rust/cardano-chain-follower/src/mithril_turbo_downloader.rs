//! Turbo Downloads for Mithril Snapshots.

use std::{
    cmp,
    ffi::OsStr,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    // process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock,
    },
};

use anyhow::{anyhow, bail};
use async_trait::async_trait;
use catalyst_types::{conversion::from_saturating, mmap_file::MemoryMapFile};
use dashmap::DashSet;
use memx::memcmp;
use mithril_client::{
    common::CompressionAlgorithm,
    file_downloader::{DownloadEvent, FileDownloader, FileDownloaderUri},
    MithrilResult,
};
use tar::{Archive, EntryType};
use tokio::{fs::create_dir_all, task::spawn_blocking};
use tracing::{debug, error};
use zstd::Decoder;

use crate::{
    mithril_snapshot_config::MithrilSnapshotConfig,
    mithril_snapshot_data::latest_mithril_snapshot_data,
    stats::{self},
    turbo_downloader::ParallelDownloadProcessor,
};

/// A snapshot downloader that accelerates Download using `aria2`.
pub struct Inner {
    /// Configuration for the snapshot sync.
    cfg: MithrilSnapshotConfig,
    /// Last hashmap/list of changed chunks from the previous download
    new_chunks: Arc<DashSet<PathBuf>>,

    /// The number of files that were new in this download.
    new_files: AtomicU64,
    /// The number of files that changed in this download.
    chg_files: AtomicU64,
    /// The total number of files in the download.
    tot_files: AtomicU64,
    /// The total size of the files extracted in the download.
    ext_size: AtomicU64,
    /// The total size of the files we deduplicated.
    dedup_size: AtomicU64,

    /// The download processor for the current file download.
    dl_handler: std::sync::OnceLock<ParallelDownloadProcessor>,
}

/// This macro is what happens every time the file is different from previous.
macro_rules! changed_file {
    ($self:ident, $rel_file:ident, $abs_file:ident, $new_size:ident) => {
        $self.chg_files.fetch_add(1, Ordering::SeqCst);
        if $abs_file.extension() == Some(OsStr::new("chunk")) {
            $self.new_chunks.insert($abs_file);
        }
    };
}

/// This macro is what happens every time we decide the file can't be deduplicated.
macro_rules! new_file {
    ($self:ident, $rel_file:ident, $abs_file:ident, $new_size:ident) => {
        $self.new_files.fetch_add(1, Ordering::SeqCst);
        if $abs_file.extension() == Some(OsStr::new("chunk")) {
            $self.new_chunks.insert($abs_file);
        }
    };
}

impl Inner {
    /// Synchronous Download and Dedup archive.
    ///
    /// Stream Downloads and Decompresses files, and deduplicates them as they are
    /// extracted from the embedded tar archive.
    ///
    /// Per Entry:
    ///   If the file is NOT to be deduplicated, OR A previous file with the same name and
    /// size does not     exist, then just extract it where its supposed to go.
    ///
    /// To Dedup, the original file is mam-mapped.
    /// The new file is extracted to an in-memory buffer.
    /// If they compare the same, the original file is `HardLinked` to the new file name.
    /// Otherwise the new file buffer is saved to disk with the new file name.
    fn dl_and_dedup(&self, _location: &str, _target_dir: &Path) -> MithrilResult<()> {
        let mut archive = self.create_archive_extractor()?;

        // Iterate the files in the archive.
        let entries = match archive.entries() {
            Ok(entries) => entries,
            Err(error) => bail!("Failed to get entries from the archive: {error}"),
        };

        let tmp_dir = self.cfg.tmp_path();
        let latest_snapshot = latest_mithril_snapshot_data(self.cfg.chain);

        for entry in entries {
            let mut entry = match entry {
                Ok(entry) => entry,
                Err(error) => bail!("Failed to get an entry from the archive: {error}"),
            };
            let rel_file = entry.path()?.to_path_buf();
            let entry_size = entry.size();

            // debug!(chain = %self.cfg.chain, "DeDup : Extracting {}:{} loc {location} target {}",
            // rel_file.to_string_lossy(), entry_size, target_dir.to_string_lossy());

            // Check if we need to extract this path or not.
            if !self.check_for_extract(&rel_file, entry.header().entry_type()) {
                continue;
            }

            // Count total files processed.
            self.tot_files.fetch_add(1, Ordering::SeqCst);

            let mut abs_file = tmp_dir.clone();
            abs_file.push(rel_file.clone());

            let mut prev_file = latest_snapshot.id().path_if_exists();
            if let Some(prev_file) = &mut prev_file {
                prev_file.push(rel_file.clone());
            }

            // debug!(chain = %self.cfg.chain, "DeDup : tmp_dir {} abs_file {} prev_file
            // {prev_file:?}", tmp_dir.to_string_lossy(), abs_file.to_string_lossy() );

            self.ext_size.fetch_add(entry_size, Ordering::SeqCst);

            // Try and deduplicate the file if we can, otherwise just extract it.
            if let Ok(prev_mmap) = Self::can_deduplicate(&rel_file, entry_size, prev_file.as_ref())
            {
                let expected_file_size = from_saturating(entry_size);
                let mut buf: Vec<u8> = Vec::with_capacity(expected_file_size);
                if entry.read_to_end(&mut buf)? != expected_file_size {
                    bail!(
                        "Failed to read file {} of size {} got {}",
                        rel_file.display(),
                        entry_size,
                        buf.len()
                    );
                }
                // Got the full file and its the expected size.  Is it different?
                if memcmp(prev_mmap.as_slice(), buf.as_slice()) == cmp::Ordering::Equal {
                    // Same so lets Hardlink it, and throw away the temp buffer.

                    // Make sure our big mmap get dropped.
                    drop(prev_mmap);

                    // File is the same, so dedup it.
                    if self.cfg.dedup_tmp(&abs_file, &latest_snapshot).is_ok() {
                        self.dedup_size.fetch_add(entry_size, Ordering::SeqCst);
                        changed_file!(self, rel_file, abs_file, entry_size);
                        drop(buf);
                        continue;
                    }
                }

                if let Err(error) = std::fs::write(&abs_file, buf) {
                    error!(chain = %self.cfg.chain, "Failed to write file {} got {}", abs_file.display(), error);
                    bail!("Failed to write file {} got {}", abs_file.display(), error);
                }
            } else {
                // No dedup, just extract it into the tmp directory as-is.
                entry.unpack_in(&tmp_dir)?;
                debug!(chain = %self.cfg.chain, "DeDup: Extracted file {rel_file:?}:{entry_size}");
            }
            new_file!(self, rel_file, abs_file, entry_size);
        }

        let Some(dl_handler) = self.dl_handler.get() else {
            bail!("Failed to get the Parallel Download processor!");
        };

        debug!(chain = %self.cfg.chain, "Download {} bytes", dl_handler.dl_size());

        stats::mithril_dl_finished(self.cfg.chain, Some(dl_handler.dl_size()));

        Ok(())
    }

    /// Create a TAR archive extractor from the downloading file and a zstd decompressor.
    fn create_archive_extractor(
        &self,
    ) -> MithrilResult<Archive<Decoder<'static, BufReader<BufReader<ParallelDownloadProcessor>>>>>
    {
        let Some(dl_handler) = self.dl_handler.get() else {
            bail!("Failed to get the Parallel Download processor!");
        };
        let buf_reader = BufReader::new(dl_handler.clone());
        let decoder = match zstd::Decoder::new(buf_reader) {
            Ok(decoder) => decoder,
            Err(error) => bail!("Failed to create ZSTD decoder: {error}"),
        };
        Ok(tar::Archive::new(decoder))
    }

    /// Check if we are supposed to extract this file from the archive or not.
    fn check_for_extract(&self, path: &Path, extract_type: EntryType) -> bool {
        if path.is_absolute() {
            error!(chain = %self.cfg.chain, "DeDup : Cannot extract an absolute path:  {:?}", path);
            return false;
        }

        if extract_type.is_dir() {
            // We don't do anything with just a path, so skip it.
            return false;
        }

        if !extract_type.is_file() {
            error!(chain  = %self.cfg.chain, "DeDup : Cannot extract a non-file: {:?}:{:?}", path, extract_type);
            return false;
        }

        true
    }

    /// Check if a given path from the archive is able to be deduplicated.
    fn can_deduplicate(
        rel_file: &Path, file_size: u64, prev_file: Option<&PathBuf>,
    ) -> MithrilResult<MemoryMapFile> {
        // Can't dedup if the current file is not de-dupable (must be immutable)
        if rel_file.starts_with("immutable") {
            // Can't dedup if we don't have a previous file to dedup against.
            if let Some(prev_file) = prev_file {
                if let Some(current_size) = get_file_size_sync(prev_file) {
                    // If the current file is not exactly the same as the previous file size, we
                    // can't dedup.
                    if file_size == current_size {
                        if let Ok(pref_file_loaded) = Self::mmap_open_sync(prev_file) {
                            if pref_file_loaded.size() == file_size {
                                return Ok(pref_file_loaded);
                            }
                        }
                    }
                }
            }
        }
        bail!("Can not deduplicate.");
    }

    /// Open a file using mmap for performance.
    fn mmap_open_sync(path: &Path) -> MithrilResult<MemoryMapFile> {
        match MemoryMapFile::try_from(path) {
            Ok(mmap_file) => Ok(mmap_file),
            Err(error) => {
                error!(error=%error, file=%path.to_string_lossy(), "Failed to open file");
                Err(error.into())
            },
        }
    }
}

/// A snapshot downloader that accelerates Download using `aria2`.
pub struct MithrilTurboDownloader {
    /// inner arc wrapped configuration
    inner: Arc<Inner>,
}

impl MithrilTurboDownloader {
    /// Constructs a new `HttpSnapshotDownloader`.
    pub fn new(cfg: MithrilSnapshotConfig) -> Self {
        // Test if the HTTP Client can properly be created.
        let dl_config = cfg.dl_config.clone().unwrap_or_default();

        let cfg = cfg.with_dl_config(dl_config);

        Self {
            inner: Arc::new(Inner {
                cfg,
                new_chunks: Arc::new(DashSet::new()),
                new_files: AtomicU64::new(0),
                chg_files: AtomicU64::new(0),
                tot_files: AtomicU64::new(0),
                ext_size: AtomicU64::new(0),
                dedup_size: AtomicU64::new(0),
                dl_handler: OnceLock::new(),
            }),
        }
    }

    /// Take the hashmap for the previous download.
    pub fn get_new_chunks(&self) -> Arc<DashSet<PathBuf>> {
        self.inner.new_chunks.clone()
    }

    /// Create directories required to exist for download to succeed.
    async fn create_directories(&self, target_dir: &Path) -> MithrilResult<()> {
        if let Err(error) = create_dir_all(target_dir).await {
            let msg = format!(
                "Target directory {} could not be created: {}",
                target_dir.to_string_lossy(),
                error
            );
            Err(anyhow!(msg.clone()).context(msg))?;
        }

        Ok(())
    }

    /// Parallel Download, Extract and Dedup the Mithril Archive.
    async fn dl_and_dedup(&self, location: &str, target_dir: &Path) -> MithrilResult<()> {
        // Get a copy of the inner data to use in the sync download task.
        let inner = self.inner.clone();
        let location = location.to_owned();
        let target_dir = target_dir.to_owned();

        // This is fully synchronous IO, so do it on a sync thread.
        let result = spawn_blocking(move || {
            stats::start_thread(
                inner.cfg.chain,
                stats::thread::name::MITHRIL_DL_DEDUP,
                false,
            );
            let result = inner.dl_and_dedup(&location, &target_dir);
            stats::stop_thread(inner.cfg.chain, stats::thread::name::MITHRIL_DL_DEDUP);
            result
        })
        .await;

        // Must always be called when download ends, regardless of the reason.
        stats::mithril_dl_finished(self.inner.cfg.chain, None);

        if let Ok(result) = result {
            return result;
        }

        bail!("Download and Dedup task failed");
    }
}

/// Get the size of a particular file.  None = failed to get size (doesn't matter why).
fn get_file_size_sync(file: &Path) -> Option<u64> {
    let Ok(metadata) = file.metadata() else {
        return None;
    };
    Some(metadata.len())
}

#[async_trait]
impl FileDownloader for MithrilTurboDownloader {
    async fn download_unpack(
        &self, location: &FileDownloaderUri, file_size: u64, target_dir: &Path,
        compression_algorithm: Option<CompressionAlgorithm>, download_event_type: DownloadEvent,
    ) -> MithrilResult<()> {
        debug!(
            location=?location,
            file_size=file_size,
            target_dir=?target_dir,
            compression_algorithm=?compression_algorithm,
            download_event_type=?download_event_type,
            "Download And Unpack Mithril."
        );

        // We only support full downloads for now.
        if !matches!(download_event_type, DownloadEvent::Full {
            download_id: _,
            digest: _
        }) {
            bail!("Unsupported Download Event Type: {:?}", download_event_type);
        }

        let location = location.as_str();

        // Probe was removed in FileDownloader, so call it directly
        self.probe(location).await?;

        self.create_directories(target_dir).await?;

        // DL Start stats set after DL actually started inside the probe call.
        self.dl_and_dedup(location, target_dir).await?;

        let tot_files = self.inner.tot_files.load(Ordering::SeqCst);
        let chg_files = self.inner.chg_files.load(Ordering::SeqCst);
        let new_files = self.inner.new_files.load(Ordering::SeqCst);

        stats::mithril_extract_finished(
            self.inner.cfg.chain,
            Some(self.inner.ext_size.load(Ordering::SeqCst)),
            self.inner.dedup_size.load(Ordering::SeqCst),
            tot_files
                .saturating_sub(chg_files)
                .saturating_sub(new_files),
            chg_files,
            new_files,
        );

        debug!("Download and Unpack finished='{location}' to '{target_dir:?}'.");

        Ok(())
    }
}

impl MithrilTurboDownloader {
    /// Set up the download.  
    /// Called `probe` as this used to exist in an earlier trait which was removed.
    async fn probe(&self, location: &str) -> MithrilResult<()> {
        debug!("Probe Snapshot location='{location}'.");
        let dl_config = self.inner.cfg.dl_config.clone().unwrap_or_default();
        let dl_processor =
            ParallelDownloadProcessor::new(location, dl_config, self.inner.cfg.chain).await?;

        // Decompress and extract and de-dupe each file in the archive.
        stats::mithril_extract_started(self.inner.cfg.chain);

        // We also immediately start downloading now.
        stats::mithril_dl_started(self.inner.cfg.chain);

        // Save the DownloadProcessor in the inner struct for use to process the downloaded data.
        if let Err(_error) = self.inner.dl_handler.set(dl_processor) {
            bail!("Failed to set the inner dl_handler. Must already be set?");
        }

        Ok(())
    }
}
