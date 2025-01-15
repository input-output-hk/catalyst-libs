//! Memory-mapped file.

use std::path::Path;

use fmmap::{MmapFile, MmapFileExt};

/// Memory-mapped file.
pub(crate) struct MemoryMapFile {
    /// The memory-mapped file.
    file: MmapFile,
    /// The size of the memory-mapped file.
    size: u64,
}

impl MemoryMapFile {
    /// Get the memory-mapped file.
    pub(crate) fn file(&self) -> &MmapFile {
        &self.file
    }

    /// Get the memory-mapped file as a slice.
    pub(crate) fn file_as_slice(&self) -> &[u8] {
        self.file().as_slice()
    }

    /// Get the size of the memory-mapped file.
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl TryFrom<&Path> for MemoryMapFile {
    type Error = fmmap::error::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // Attempt to open the file with memory mapping options
        match MmapFile::open_with_options(path, fmmap::Options::new().read(true).populate()) {
            Ok(file) => {
                let len = file.len() as u64;
                Ok(MemoryMapFile { file, size: len })
            },
            Err(error) => Err(error),
        }
    }
}
