//! Hasher for the Sparse Merkle Tree module.
//!
//! Uses the blake3 hash internally.

use sparse_merkle_tree::H256;

/// Hasher
pub(super) struct Hasher(blake3::Hasher);

impl Hasher {
    /// Create a new hasher
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }

    /// Utility function to quickly hash bytes
    pub(super) fn hash(data: &[u8]) -> H256 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        H256::from(*hash.as_bytes())
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl sparse_merkle_tree::traits::Hasher for Hasher {
    fn write_h256(
        &mut self,
        h: &H256,
    ) {
        self.0.update(h.as_slice());
    }

    fn write_byte(
        &mut self,
        b: u8,
    ) {
        self.0.update(&[b][..]);
    }

    fn finish(self) -> H256 {
        let hash = self.0.finalize();
        let bytes = hash.as_bytes();
        (*bytes).into()
    }
}
