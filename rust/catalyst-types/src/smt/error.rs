//! Error types for Sparse Merkle Tree Module

use thiserror::Error;

/// Error type
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid key prefix length
    #[error("invalid key prefix length")]
    InvalidKeyPrefixLength,
    /// Requested slice height is too large
    #[error("slice height too large (allowed max: {allowed_max})")]
    SliceHeightTooLarge {
        /// Maximum allowed slice height
        allowed_max: u8,
    },
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct MerkleTreeError(#[from] sparse_merkle_tree::error::Error);
