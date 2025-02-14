//! Memory-mapped file.

use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use fmmap::{MmapFile, MmapFileExt};
use once_cell::sync::Lazy;
use serde::Serialize;
use tracing::error;

/// Memory-mapped file.
pub struct MemoryMapFile {
    /// The memory-mapped file.
    file: MmapFile,
    /// The size of the memory-mapped file.
    size: u64,
}

/// Global statistic for memory-mapped files.
static MEMMAP_FILE_STATS: Lazy<Arc<RwLock<MemMapFileStat>>> =
    Lazy::new(|| Arc::new(RwLock::new(MemMapFileStat::default())));

/// Memory-mapped file statistic.
#[derive(Debug, Default, Clone, Serialize)]
pub struct MemMapFileStat {
    /// A counter for the number of memory-mapped files.
    file_count: u64,
    /// The total size of memory-mapped files.
    total_size: u64,
    /// The amount of time that memory-mapped files have been dropped.
    drop_count: u64,
    /// The total size of memory-mapped files that have been dropped.
    drop_size: u64,
    /// A count of errors encountered.
    error_count: u64,
}

impl MemMapFileStat {
    /// Get the global memory-mapped file statistics.
    /// Return default statistics if the mutex is poisoned.
    pub fn current() -> MemMapFileStat {
        if let Ok(stat) = MEMMAP_FILE_STATS.read() {
            stat.clone()
        } else {
            error!("RwLock read poisoned, failed to read memory-mapped file statistics.");
            MemMapFileStat::default()
        }
    }

    /// Get the statistic file count.
    #[must_use]
    pub fn file_count(&self) -> u64 {
        self.file_count
    }

    /// Get the statistic total size.
    #[must_use]
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get the statistic drop count.
    #[must_use]
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }

    /// Get the statistic drop size.
    #[must_use]
    pub fn drop_size(&self) -> u64 {
        self.drop_size
    }

    /// Get the statistic error count.
    #[must_use]
    pub fn error_count(&self) -> u64 {
        self.error_count
    }

    /// Update the global stats when a file is created.
    fn update_create_stat(size: u64) {
        if let Ok(mut stat) = MEMMAP_FILE_STATS.write() {
            stat.file_count = stat.file_count.saturating_add(1);
            stat.total_size = stat.total_size.saturating_add(size);
        } else {
            error!(
                "RwLock write poisoned, failed to update created memory-mapped file statistics."
            );
        }
    }

    /// Update the global stats when a file is dropped.
    fn update_drop_stat(size: u64) {
        if let Ok(mut stat) = MEMMAP_FILE_STATS.write() {
            stat.drop_count = stat.drop_count.saturating_add(1);
            stat.drop_size = stat.drop_size.saturating_add(size);
        } else {
            error!(
                "RwLock write poisoned, failed to update dropped memory-mapped file statistics."
            );
        }
    }

    /// Update the global error count when an error occurs.
    fn update_err_stat() {
        if let Ok(mut stat) = MEMMAP_FILE_STATS.write() {
            stat.error_count = stat.error_count.saturating_add(1);
        } else {
            error!("RwLock write poisoned, failed to update error memory-mapped file statistics.");
        }
    }
}

impl MemoryMapFile {
    /// Get the size of the memory-mapped file.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the memory-mapped file as a slice.
    pub fn as_slice(&self) -> &[u8] {
        self.file.as_slice()
    }
}

impl Drop for MemoryMapFile {
    fn drop(&mut self) {
        MemMapFileStat::update_drop_stat(self.size);
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
                MemMapFileStat::update_create_stat(len);
                Ok(memory_map_file)
            },
            Err(error) => {
                MemMapFileStat::update_err_stat();
                Err(error)
            },
        }
    }
}
