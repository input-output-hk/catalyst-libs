//! Thread names

/// Chain Sync.
pub(crate) const CHAIN_SYNC: &str = "ChainSync";
/// Wait for Sync Ready.
pub(crate) const WAIT_FOR_SYNC_READY: &str = "WaitForSyncReady";
/// Live Sync Backfill and Purge.
pub(crate) const LIVE_SYNC_BACKFILL_AND_PURGE: &str = "LiveSyncBackfillAndPurge";
/// Mithril Iterator.
pub(crate) const MITHRIL_ITERATOR: &str = "MithrilIterator";
/// Background Mithril Snapshot Updater.
pub(crate) const MITHRIL_SNAPSHOT_UPDATER: &str = "MithrilSnapshotUpdater";
/// Mithril compute snapshot.
pub(crate) const COMPUTE_SNAPSHOT_MSG: &str = "ComputeSnapshotMsg";
/// Background Mithril Snapshot Validator.
pub(crate) const VALIDATE_MITHRIL_SNAPSHOT: &str = "ValidateMithrilSnapshot";
/// Mithril Downloader, Dl and Dedup.
pub(crate) const MITHRIL_DL_DEDUP: &str = "MithrilDlDedup";
/// Parallel Download Processor Worker.
pub(crate) const PARALLEL_DL_WORKER: &str = "ParallelDlWorker";
/// Parallel Download Processor Get Content Length.
pub(crate) const PARALLEL_DL_GET_CONTENT_LENGTH: &str = "ParallelDlGetContentLength";
