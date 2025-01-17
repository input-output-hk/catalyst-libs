//! Memory-mapped file.

use core::fmt;
use fmmap::{MmapFile, MmapFileExt};
use once_cell::sync::Lazy;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

/// Memory-mapped file.
pub struct MemoryMapFile {
    /// The memory-mapped file.
    file: MmapFile,
    /// The size of the memory-mapped file.
    size: u64,
}
/// Global statistic for memory-mapped files.
static MEMMAP_FILE_STATS: Lazy<MemMapFileStat> = Lazy::new(MemMapFileStat::default);

/// Memory-mapped file statistic.
#[derive(Debug, Default, Clone, Serialize)]
pub struct MemMapFileStat(Arc<MemMapFileStatInner>);

impl MemMapFileStat {
    /// Get the statistic file count.
    #[must_use]
    pub fn file_count(&self) -> u64 {
        self.0.file_count.load(Ordering::SeqCst)
    }

    /// Get the statistic total size.
    #[must_use]
    pub fn total_size(&self) -> u64 {
        self.0.total_size.load(Ordering::SeqCst)
    }

    /// Get the statistic drop count.
    #[must_use]
    pub fn drop_count(&self) -> u64 {
        self.0.drop_count.load(Ordering::SeqCst)
    }

    /// Get the statistic drop size.
    #[must_use]
    pub fn drop_size(&self) -> u64 {
        self.0.drop_size.load(Ordering::SeqCst)
    }

    /// Get the statistic error count.
    #[must_use]
    pub fn error_count(&self) -> u64 {
        self.0.error_count.load(Ordering::SeqCst)
    }
}

/// Internal structure to hold stats.
struct MemMapFileStatInner {
    /// A counter for the number of memory-mapped files.
    file_count: AtomicU64,
    /// The total size of memory-mapped files.
    total_size: AtomicU64,
    /// The amount of time that memory-mapped files have been dropped.
    drop_count: AtomicU64,
    /// The total size of memory-mapped files that have been dropped.
    drop_size: AtomicU64,
    /// A count of errors encountered.
    error_count: AtomicU64,
}

impl MemoryMapFile {
    /// Get the memory-mapped file.
    pub fn file(&self) -> &MmapFile {
        &self.file
    }

    /// Get the memory-mapped file as a slice.
    pub fn file_as_slice(&self) -> &[u8] {
        self.file().as_slice()
    }

    /// Get the size of the memory-mapped file.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the global memory-mapped file statistics.
    #[must_use]
    pub fn stat() -> &'static MemMapFileStat {
        &MEMMAP_FILE_STATS
    }

    /// Update the global stats when a file is created.
    fn update_create_stat(&self) {
        MEMMAP_FILE_STATS
            .0
            .file_count
            .fetch_add(1, Ordering::SeqCst);
        MEMMAP_FILE_STATS
            .0
            .total_size
            .fetch_add(self.size, Ordering::SeqCst);
    }

    /// Update the global stats when a file is dropped.
    fn update_drop_stat(&self) {
        MEMMAP_FILE_STATS
            .0
            .drop_count
            .fetch_add(1, Ordering::SeqCst);
        MEMMAP_FILE_STATS
            .0
            .drop_size
            .fetch_add(self.size, Ordering::SeqCst);
    }

    /// Update the global error count when an error occurs.
    pub fn update_err_stat() {
        MEMMAP_FILE_STATS
            .0
            .error_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for MemoryMapFile {
    fn drop(&mut self) {
        self.update_drop_stat();
    }
}

impl TryFrom<&Path> for MemoryMapFile {
    type Error = fmmap::error::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // Attempt to open the file with memory mapping options
        match MmapFile::open_with_options(path, fmmap::Options::new().read(true).populate()) {
            Ok(file) => {
                let len = file.len() as u64;
                let memory_map_file = MemoryMapFile { file, size: len };
                memory_map_file.update_create_stat();
                Ok(memory_map_file)
            },
            Err(error) => {
                Self::update_err_stat();
                Err(error)
            },
        }
    }
}

impl Default for MemMapFileStatInner {
    fn default() -> Self {
        Self {
            file_count: AtomicU64::new(0),
            total_size: AtomicU64::new(0),
            drop_count: AtomicU64::new(0),
            drop_size: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
}

impl fmt::Debug for MemMapFileStatInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemMapFileStat")
            .field("file_count", &self.file_count.load(Ordering::SeqCst))
            .field("total_size", &self.total_size.load(Ordering::SeqCst))
            .field("drop_count", &self.drop_count.load(Ordering::SeqCst))
            .field("drop_size", &self.drop_size.load(Ordering::SeqCst))
            .field("error_count", &self.error_count.load(Ordering::SeqCst))
            .finish()
    }
}

impl Serialize for MemMapFileStatInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MemMapFileStat", 5)?;

        state.serialize_field("file_count", &self.file_count.load(Ordering::SeqCst))?;
        state.serialize_field("total_size", &self.total_size.load(Ordering::SeqCst))?;
        state.serialize_field("drop_count", &self.drop_count.load(Ordering::SeqCst))?;
        state.serialize_field("drop_size", &self.drop_size.load(Ordering::SeqCst))?;
        state.serialize_field("error_count", &self.error_count.load(Ordering::SeqCst))?;
        state.end()
    }
}
