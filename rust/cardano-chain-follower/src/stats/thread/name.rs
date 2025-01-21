//! Thread names

/// Chain Sync.
pub(crate) const CHAIN_SYNC: &str = "Async:ChainSync";
/// Wait for Sync Ready.
pub(crate) const WAIT_FOR_SYNC_READY: &str = "Async:WaitForSyncReady";
/// Live Sync Backfill and Purge.
pub(crate) const LIVE_SYNC_BACKFILL_AND_PURGE: &str = "Async:LiveSyncBackfillAndPurge";
/// Mithril Iterator.
pub(crate) const MITHRIL_ITERATOR: &str = "MithrilIterator";
/// Background Mithril Snapshot Updater.
pub(crate) const MITHRIL_SNAPSHOT_UPDATER: &str = "Async:MithrilSnapshotUpdater";
/// Mithril compute snapshot.
pub(crate) const COMPUTE_SNAPSHOT_MSG: &str = "Async:ComputeSnapshotMsg";
/// Background Mithril Snapshot Validator.
pub(crate) const VALIDATE_MITHRIL_SNAPSHOT: &str = "Async:ValidateMithrilSnapshot";
/// Mithril Downloader, Dl and Dedup.
pub(crate) const MITHRIL_DL_DEDUP: &str = "MithrilDlDedup";
/// Parallel Download Processor Worker.
pub(crate) const PARALLEL_DL_WORKER: &str = "ParallelDlWorker";
