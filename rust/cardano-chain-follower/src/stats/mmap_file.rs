//! Memory-mapped file statistics.

use core::fmt;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Memory-mapped file statistics.
#[derive(Debug, Default, Clone, Serialize)]
pub struct MMapFileStat(Arc<InnerMMapFileStat>);

/// Inner memory-mapped file statistics.
struct InnerMMapFileStat {
    /// A counter for the number of memory-mapped files.
    file_counter: AtomicU64,
    /// The total size of memory-mapped files.
    total_size: AtomicU64,
    /// A boolean value indicating whether the memory-mapped files have been dropped.
    is_drop: AtomicBool,
}

impl Serialize for InnerMMapFileStat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut state = serializer.serialize_struct("MMapFileStat", 3)?;

        state.serialize_field("file_counter", &self.file_counter.load(Ordering::SeqCst))?;
        state.serialize_field("total_size", &self.total_size.load(Ordering::SeqCst))?;
        state.serialize_field("is_drop", &self.is_drop.load(Ordering::SeqCst))?;
        state.end()
    }
}

impl fmt::Debug for InnerMMapFileStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InnerMMapFileStat")
            .field("file_counter", &self.file_counter.load(Ordering::SeqCst))
            .field("total_size", &self.total_size.load(Ordering::SeqCst))
            .field("is_drop", &self.is_drop.load(Ordering::SeqCst))
            .finish()
    }
}

impl Default for InnerMMapFileStat {
    fn default() -> Self {
        Self {
            file_counter: AtomicU64::new(0),
            total_size: AtomicU64::new(0),
            is_drop: AtomicBool::new(false),
        }
    }
}

impl MMapFileStat {
    /// Increment the memory-mapped file counter.
    pub fn incr_file_counter(&self) {
        self.0.file_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Update the total size of memory-mapped files.
    pub fn update_total_size(&self, size: u64) {
        self.0.total_size.fetch_add(size, Ordering::SeqCst);
    }

    /// Set the memory-mapped files as dropped.
    pub fn set_is_drop(&self) {
        self.0.is_drop.store(true, Ordering::SeqCst);
    }

    /// Get the memory-mapped file counter.
    pub fn file_counter(&self) -> u64 {
        self.0.file_counter.load(Ordering::SeqCst)
    }

    /// Get the total size of memory-mapped files.
    pub fn total_size(&self) -> u64 {
        self.0.total_size.load(Ordering::SeqCst)
    }

    /// Get the boolean value indicating whether the memory-mapped files have been
    /// dropped.
    pub fn is_drop(&self) -> bool {
        self.0.is_drop.load(Ordering::SeqCst)
    }
}
